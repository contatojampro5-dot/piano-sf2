"""Junta vários SF2 num SF2 só (um timbre por arquivo de entrada), para o Piano SF2 abrir tudo de uma vez.

Uso: python3 tools/montar_pacote.py saida.sf2 "Nome 1=arquivo1.sf2" "Nome 2=arquivo2.sf2" ...
Cada arquivo de entrada vira timbres no banco 0, na ordem (programa 0, 1, 2...). Se o arquivo
tiver vários timbres, todos entram (com o nome do arquivo na frente).
"""
import struct
import sys


def chunks(data, off, end):
    while off + 8 <= end:
        cid = data[off:off + 4]
        size = struct.unpack_from('<I', data, off + 4)[0]
        yield cid, off + 8, size
        off += 8 + size + (size & 1)


def read_sf2(path):
    data = open(path, 'rb').read()
    assert data[:4] == b'RIFF' and data[8:12] == b'sfbk', path
    out = {}
    for cid, off, size in chunks(data, 12, len(data)):
        if cid != b'LIST':
            continue
        kind = data[off:off + 4]
        for sub, soff, ssize in chunks(data, off + 4, off + size):
            out[(kind, sub)] = data[soff:soff + ssize]
    return out


def records(blob, size):
    return [blob[i:i + size] for i in range(0, len(blob) - len(blob) % size, size)]


def main():
    out_path = sys.argv[1]
    entries = []
    for arg in sys.argv[2:]:
        name, path = arg.split('=', 1)
        entries.append((name, path))

    smpl = bytearray()
    phdr, pbag, pmod, pgen = [], [], [], []
    inst, ibag, imod, igen = [], [], [], []
    shdr = []
    program = 0

    for name, path in entries:
        sf = read_sf2(path)
        s_base = len(smpl) // 2
        smpl += sf[(b'sdta', b'smpl')]
        smpl += b'\x00' * 92  # 46 amostras de silêncio entre arquivos (pede a especificação)

        # --- amostras (shdr): desloca posições e ligações
        sh = records(sf[(b'pdta', b'shdr')], 46)[:-1]  # sem o terminal "EOS"
        sh_base = len(shdr)
        for r in sh:
            nm = r[:20]
            start, end, ls, le, rate = struct.unpack_from('<5I', r, 20)
            pitch, corr = r[40], r[41]
            link, typ = struct.unpack_from('<HH', r, 42)
            shdr.append(nm + struct.pack('<5I', start + s_base, end + s_base, ls + s_base, le + s_base, rate)
                        + bytes([pitch, corr]) + struct.pack('<HH', (link + sh_base) if typ in (2, 4, 8) else link, typ))

        # --- instrumentos (inst/ibag/imod/igen): sampleID (53) aponta para o shdr novo
        ig = records(sf[(b'pdta', b'igen')], 4)
        im = records(sf[(b'pdta', b'imod')], 10)
        ib = records(sf[(b'pdta', b'ibag')], 4)
        ins = records(sf[(b'pdta', b'inst')], 22)
        ig_base, im_base, ib_base, in_base = len(igen), len(imod), len(ibag), len(inst)
        for r in ig[:-1]:
            op, amt = struct.unpack('<HH', r)
            if op == 53:
                amt += sh_base
            igen.append(struct.pack('<HH', op, amt))
        imod += im[:-1]
        for r in ib[:-1]:
            g, m = struct.unpack('<HH', r)
            ibag.append(struct.pack('<HH', g + ig_base, m + im_base))
        for r in ins[:-1]:
            bag = struct.unpack_from('<H', r, 20)[0]
            inst.append(r[:20] + struct.pack('<H', bag + ib_base))

        # --- timbres (phdr/pbag/pmod/pgen): instrument (41) aponta para o inst novo
        pg = records(sf[(b'pdta', b'pgen')], 4)
        pm = records(sf[(b'pdta', b'pmod')], 10)
        pb = records(sf[(b'pdta', b'pbag')], 4)
        ph = records(sf[(b'pdta', b'phdr')], 38)
        pg_base, pm_base, pb_base = len(pgen), len(pmod), len(pbag)
        for r in pg[:-1]:
            op, amt = struct.unpack('<HH', r)
            if op == 41:
                amt += in_base
            pgen.append(struct.pack('<HH', op, amt))
        pmod += pm[:-1]
        for r in pb[:-1]:
            g, m = struct.unpack('<HH', r)
            pbag.append(struct.pack('<HH', g + pg_base, m + pm_base))
        real = ph[:-1]
        # timbres do arquivo em ordem de banco/programa (o primeiro é o principal)
        real.sort(key=lambda r: struct.unpack_from('<HH', r, 22)[::-1])
        for k, r in enumerate(real):
            bag = struct.unpack_from('<H', r, 24)[0]
            orig = r[:20].split(b'\x00')[0].decode('latin-1').strip()
            label = name if len(real) == 1 else f'{name} {orig.split()[-1] if orig else k + 1}'
            nm = label.encode('latin-1', 'replace')[:19].ljust(20, b'\x00')
            phdr.append(nm + struct.pack('<HHH', program, 0, bag + pb_base) + r[26:38])
            program += 1

    # terminais
    phdr.append(b'EOP'.ljust(20, b'\x00') + struct.pack('<HHH', 0, 0, len(pbag)) + b'\x00' * 12)
    pbag.append(struct.pack('<HH', len(pgen), len(pmod)))
    pmod.append(b'\x00' * 10)
    pgen.append(b'\x00' * 4)
    inst.append(b'EOI'.ljust(20, b'\x00') + struct.pack('<H', len(ibag)))
    ibag.append(struct.pack('<HH', len(igen), len(imod)))
    imod.append(b'\x00' * 10)
    igen.append(b'\x00' * 4)
    shdr.append(b'EOS'.ljust(20, b'\x00') + b'\x00' * 26)

    def chunk(cid, data):
        data = bytes(data)
        if len(data) & 1:  # tamanho par (alguns leitores não pulam o byte de enchimento)
            data += b'\x00'
        return cid + struct.pack('<I', len(data)) + data + (b'\x00' if len(data) & 1 else b'')

    def lst(kind, parts):
        body = kind + b''.join(parts)
        return b'LIST' + struct.pack('<I', len(body)) + body

    info = lst(b'INFO', [
        chunk(b'ifil', struct.pack('<HH', 2, 1)),
        chunk(b'isng', b'EMU8000\x00'),
        chunk(b'INAM', b'Pacote Igreja - Piano SF2\x00'),
        chunk(b'ICMT', b'Sons livres (FreePats, CC0/CC-BY). Veja CREDITOS.txt\x00'),
    ])
    sdta = lst(b'sdta', [chunk(b'smpl', smpl)])
    pdta = lst(b'pdta', [chunk(b'phdr', b''.join(phdr)), chunk(b'pbag', b''.join(pbag)), chunk(b'pmod', b''.join(pmod)),
                         chunk(b'pgen', b''.join(pgen)), chunk(b'inst', b''.join(inst)), chunk(b'ibag', b''.join(ibag)),
                         chunk(b'imod', b''.join(imod)), chunk(b'igen', b''.join(igen)), chunk(b'shdr', b''.join(shdr))])
    body = b'sfbk' + info + sdta + pdta
    with open(out_path, 'wb') as f:
        f.write(b'RIFF' + struct.pack('<I', len(body)) + body)
    print(f'{out_path}: {program} timbres, {len(body) / 1e6:.1f} MB')


if __name__ == '__main__':
    main()
