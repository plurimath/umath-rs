#!/usr/bin/env python3
"""Verify the frozen bundle and reproduce committed models with the pinned generator."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_files():
    upstream = json.loads((ROOT / 'UPSTREAM.json').read_text())
    bundle = ROOT / upstream['model_producer']['bundle']
    assert sha256(bundle) == upstream['model_producer']['sha256'], 'bundle checksum'
    generated = ROOT / 'src/generated'
    manifest = json.loads((generated / 'manifest.json').read_text())
    for file in manifest['files']:
        assert 'sha256:' + sha256(generated / file['path']) == file['sha256'], file['path']
    return upstream, bundle, generated


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('generator', nargs='?', type=Path)
    parser.add_argument('--verify-files', action='store_true')
    args = parser.parse_args()
    upstream, bundle, generated = verify_files()
    if args.verify_files:
        print('Frozen bundle and four generated file hashes match')
        return
    if args.generator is None:
        parser.error('generator checkout is required unless --verify-files is set')
    generator = args.generator.resolve()
    revision = subprocess.check_output(['git', '-C', str(generator), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == upstream['model_generator']['commit'], 'generator revision'
    with tempfile.TemporaryDirectory(prefix='umath-regenerated-') as temporary:
        output = Path(temporary)
        subprocess.run([
            'cargo', 'run', '--locked', '--manifest-path', str(generator / 'Cargo.toml'),
            '--', 'generate', '--input', str(bundle), '--output', str(output),
        ], check=True, cwd=ROOT)
        expected = {p.name for p in generated.iterdir() if p.is_file()}
        assert {p.name for p in output.iterdir()} == expected, 'generated file set'
        for name in sorted(expected):
            assert (output / name).read_bytes() == (generated / name).read_bytes(), name
    print('Pinned generator exactly reproduces all committed model files and manifest')


if __name__ == '__main__':
    main()
