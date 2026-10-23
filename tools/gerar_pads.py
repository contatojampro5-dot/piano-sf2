"""Gera os pads do Piano SF2 (12 tons × 2 estilos), sem terça (serve maior e menor), em loop de 30 s.
Uso: python3 gerar_pads.py <estilo escuro|coral> <tom 0..11> <saida.wav>"""
import sys, wave, numpy as np
from scipy.signal import butter, sosfilt, fftconvolve
SR = 44100

def hz(m): return 440 * 2 ** ((m - 69) / 12)

def saw(f, t, rng, nh, vib=None):
    ph = 2 * np.pi * f * t if vib is None else 2 * np.pi * f * np.cumsum(vib) / SR
    out = np.zeros_like(t)
    for n in range(1, nh + 1):
        out += np.sin(n * ph + rng.uniform(0, 6.28)) / n
    return out

def reverb(L, R, rng, secs=7.0, decay=1.6, lp=7000):
    n = int(SR * secs); tt = np.arange(n) / SR
    sos = butter(2, lp, 'low', fs=SR, output='sos')
    irL = sosfilt(sos, rng.standard_normal(n) * np.exp(-tt / decay))
    irR = sosfilt(sos, rng.standard_normal(n) * np.exp(-tt / decay))
    wl = fftconvolve(L, irL)[:len(L)]; wr = fftconvolve(R, irR)[:len(R)]
    return wl / np.abs(wl).max(), wr / np.abs(wr).max()

def make_loop(L, R, loop, xf):
    start = int(SR * 2.5); ln = int(SR * loop); x = int(SR * xf)
    out = np.stack([L, R], 1)[start:start + ln + x]
    fade = np.linspace(0, 1, x)[:, None]
    body = out[:ln].copy()
    body[:x] = out[ln:ln + x] * np.sqrt(1 - fade) + out[:x] * np.sqrt(fade)
    return body

def escuro(root, rng, t, loop):
    L = np.zeros_like(t); R = np.zeros_like(t)
    notes = [(root - 12, .45), (root, .55), (root + 7, .42), (root + 12, .3), (root + 19, .12)]
    for m, g in notes:
        f = hz(m)
        for v, det in enumerate(np.linspace(-8, 8, 5)):
            ff = f * 2 ** (det / 1200)
            s = saw(ff, t, rng, nh=min(18, max(1, int(3500 / ff))))
            amp = g * (1 + .25 * np.sin(2 * np.pi * rng.uniform(.04, .1) * t + rng.uniform(0, 6.28)))
            p = v / 4
            L += s * amp * np.cos(p * np.pi / 2); R += s * amp * np.sin(p * np.pi / 2)
    for m, g in [(root + 24, .05), (root + 31, .03)]:
        f = hz(m)
        env = np.clip(np.sin(2 * np.pi * rng.uniform(.03, .06) * t + rng.uniform(0, 6.28)), 0, 1) ** 2
        L += g * env * np.sin(2 * np.pi * f * t); R += g * env * np.sin(2 * np.pi * f * 1.002 * t + 1)
    dark = butter(4, 650, 'low', fs=SR, output='sos'); open_ = butter(4, 1700, 'low', fs=SR, output='sos')
    mix = (.5 + .5 * np.sin(2 * np.pi * t / loop * 2)) * .8
    L = sosfilt(dark, L) * (1 - mix) + sosfilt(open_, L) * mix
    R = sosfilt(dark, R) * (1 - mix) + sosfilt(open_, R) * mix
    wl, wr = reverb(L, R, rng, decay=1.9, lp=4500)
    return L, R, wl, wr, .4, .7

# Formantes de vogais (Hz, ganho, largura): "ah" e "oh"
VOWELS = {'a': [(700, 1.0, 110), (1150, .55, 120), (2700, .2, 180)],
          'o': [(480, 1.0, 90), (850, .5, 110), (2600, .12, 170)]}

CHEST = butter(2, 320, 'low', fs=SR, output='sos')

def formant(x, vowel):
    # "peito": a nota de baixo e os primeiros harmônicos (o grave do coro)
    out = 0.9 * sosfilt(CHEST, x)
    for fc, g, bw in VOWELS[vowel]:
        lo, hi = max(40, fc - bw), fc + bw
        out += g * sosfilt(butter(2, [lo, hi], 'band', fs=SR, output='sos'), x)
    return out

def coral(root, rng, t, loop):
    L = np.zeros_like(t); R = np.zeros_like(t)
    # graves e médios: raiz grave (baixos), raiz e quinta (tenores), oitava e quinta acima (contraltos)
    notes = [(root - 12, .5), (root, .6), (root + 7, .5), (root + 12, .42), (root + 19, .25)]
    morph = .5 + .5 * np.sin(2 * np.pi * t / loop)  # "aah" ↔ "ooh", periódico no loop
    for m, g in notes:
        f = hz(m)
        for v in range(6):  # 6 cantores por nota
            det = rng.uniform(-14, 14)
            rate = rng.uniform(4.6, 5.6); depth = rng.uniform(.002, .004)
            vib = f * 2 ** (det / 1200) * (1 + depth * np.sin(2 * np.pi * rate * t + rng.uniform(0, 6.28)))
            src = saw(f, t, rng, nh=min(40, max(1, int(6000 / f))), vib=vib)
            s = formant(src, 'a') * (1 - morph) + formant(src, 'o') * morph
            breath = 1 + .18 * np.sin(2 * np.pi * rng.uniform(.05, .12) * t + rng.uniform(0, 6.28))
            p = rng.uniform(.1, .9)
            L += s * g * breath * np.cos(p * np.pi / 2); R += s * g * breath * np.sin(p * np.pi / 2)
    # um pouco de "ar" de respiração do coro
    air = butter(2, [2000, 6000], 'band', fs=SR, output='sos')
    lvl = .015 * max(np.abs(L).max(), np.abs(R).max())
    L += lvl * sosfilt(air, rng.standard_normal(len(t))); R += lvl * sosfilt(air, rng.standard_normal(len(t)))
    lp = butter(2, 3800, 'low', fs=SR, output='sos'); L = sosfilt(lp, L); R = sosfilt(lp, R)
    wl, wr = reverb(L, R, rng, decay=1.8, lp=6000)
    return L, R, wl, wr, .4, .75

def render(style, key, loop=30, xf=6):
    rng = np.random.default_rng(100 + key + (50 if style == 'coral' else 0))
    root = 48 + key if key <= 6 else 36 + key   # C3..F#3 e G2..B2: tudo na mesma região
    t = np.arange(int(SR * (loop + xf + 3))) / SR
    L, R, wl, wr, dry, wet = (escuro if style == 'escuro' else coral)(root, rng, t, loop)
    hp = butter(2, 40, 'high', fs=SR, output='sos'); L = sosfilt(hp, L); R = sosfilt(hp, R)
    n = max(np.abs(L).max(), np.abs(R).max())
    L = dry * L / n + wet * wl; R = dry * R / n + wet * wr
    body = make_loop(L, R, loop, xf)
    return body

if __name__ == '__main__':
    style, key, path = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    body = render(style, key)
    # mesma "altura" de som para todos: RMS alvo, com teto de pico
    rms = np.sqrt((body ** 2).mean()); body *= 0.16 / rms
    body = 0.9 * np.tanh(body / 0.9)  # segura picos sem mudar o volume médio
    with wave.open(path, 'wb') as w:
        w.setnchannels(2); w.setsampwidth(2); w.setframerate(SR)
        w.writeframes((np.clip(body, -1, 1) * 32767).astype('<i2').tobytes())
