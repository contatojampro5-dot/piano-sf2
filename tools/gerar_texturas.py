"""Gera as texturas-base do "Pad Vivo" (motor granular): uma nota só (C3), seca, 8 s, mono.
O motor do programa picota isso em grãos, transpõe para qualquer tom e mistura com o reverb de igreja.
Uso: python3 gerar_texturas.py <coro|cordas|cristal> <saida.wav>"""
import sys, wave, numpy as np
from scipy.signal import butter, sosfilt

SR = 44100
ROOT = 48  # C3

def hz(m): return 440 * 2 ** ((m - 69) / 12)

def saw(phase, nh):
    out = np.zeros_like(phase)
    for n in range(1, nh + 1):
        out += np.sin(n * phase) / n
    return out

def bp(x, lo, hi, order=2):
    return sosfilt(butter(order, [lo, hi], 'band', fs=SR, output='sos'), x)

def coro(t, rng):
    f = hz(ROOT)
    out = np.zeros_like(t)
    for v in range(10):  # 10 cantores na mesma nota (com oitava abaixo em alguns)
        oct_ = 0.5 if v < 3 else 1.0
        det = rng.uniform(-12, 12)
        vib = 1 + rng.uniform(.002, .0045) * np.sin(2 * np.pi * rng.uniform(4.5, 5.8) * t + rng.uniform(0, 6.3))
        ph = 2 * np.pi * np.cumsum(f * oct_ * 2 ** (det / 1200) * vib) / SR + rng.uniform(0, 6.3)
        src = saw(ph, int(7000 / (f * oct_)))
        # vogal "ah" → "oh" devagar
        m = .5 + .5 * np.sin(2 * np.pi * t / 8 + v)
        ah = bp(src, 600, 900) + .5 * bp(src, 1050, 1300) + .18 * bp(src, 2500, 3000)
        oh = bp(src, 400, 600) + .45 * bp(src, 750, 950) + .1 * bp(src, 2400, 2800)
        chest = .8 * sosfilt(butter(2, 330, 'low', fs=SR, output='sos'), src)
        amp = 1 + .2 * np.sin(2 * np.pi * rng.uniform(.1, .3) * t + rng.uniform(0, 6.3))
        out += (ah * (1 - m) + oh * m + chest) * amp
    breath = bp(rng.standard_normal(len(t)), 1800, 6000) * .02 * np.abs(out).max()
    return out + breath

def cordas(t, rng):
    f = hz(ROOT)
    out = np.zeros_like(t)
    for v in range(8):
        oct_ = [0.5, 1, 1, 1, 1, 2, 2, 1][v]
        det = rng.uniform(-9, 9)
        vib = 1 + .0035 * np.sin(2 * np.pi * rng.uniform(5.0, 6.2) * t + rng.uniform(0, 6.3))
        ph = 2 * np.pi * np.cumsum(f * oct_ * 2 ** (det / 1200) * vib) / SR + rng.uniform(0, 6.3)
        s = saw(ph, int(9000 / (f * oct_)))
        # "arco": ruído de crina que acompanha o som
        bow = bp(rng.standard_normal(len(t)), 1500, 5000) * .05
        amp = 1 + .15 * np.sin(2 * np.pi * rng.uniform(.15, .4) * t + rng.uniform(0, 6.3))
        out += (s + bow) * amp
    # corpo de madeira: ressonâncias
    body = bp(out, 250, 500) * .6 + bp(out, 900, 1400) * .4 + bp(out, 2500, 3500) * .25
    return sosfilt(butter(2, 5500, 'low', fs=SR, output='sos'), out * .5 + body)

def cristal(t, rng):
    f = hz(ROOT)
    out = np.zeros_like(t)
    # toques de sino/vidro (FM) espalhados, com cauda longa
    for k in range(14):
        t0 = rng.uniform(0, 7.0)
        mult = rng.choice([1, 2, 2, 4])
        idx = np.clip(t - t0, 0, None)
        env = np.where(t >= t0, np.exp(-idx / rng.uniform(1.2, 2.5)) * (1 - np.exp(-idx * 300)), 0)
        car = 2 * np.pi * f * mult * idx
        mod = rng.uniform(1.5, 3.5) * np.exp(-idx / .6) * np.sin(2 * np.pi * f * mult * 3.5 * idx)
        out += env * np.sin(car + mod) / mult ** .5
    # "vidro friccionado": senos contínuos que respiram
    for mult, g in [(1, .35), (2, .25), (3, .12), (4, .1), (6, .05)]:
        sw = .6 + .4 * np.sin(2 * np.pi * rng.uniform(.08, .2) * t + rng.uniform(0, 6.3))
        out += g * sw * np.sin(2 * np.pi * f * mult * t * (1 + rng.uniform(-.001, .001)))
    return out

if __name__ == '__main__':
    style, path = sys.argv[1], sys.argv[2]
    rng = np.random.default_rng({'coro': 11, 'cordas': 22, 'cristal': 33}[style])
    t = np.arange(int(SR * 8.0)) / SR
    x = {'coro': coro, 'cordas': cordas, 'cristal': cristal}[style](t, rng)
    x = sosfilt(butter(2, 35, 'high', fs=SR, output='sos'), x)
    fade = int(SR * .05)
    x[:fade] *= np.linspace(0, 1, fade); x[-fade:] *= np.linspace(1, 0, fade)
    x = .9 * x / np.abs(x).max()
    with wave.open(path, 'wb') as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR)
        w.writeframes((x * 32767).astype('<i2').tobytes())
