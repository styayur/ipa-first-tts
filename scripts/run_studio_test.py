# SPDX-License-Identifier: MIT
"""Run a UI test against an exclusively owned real Rust server, without shell children.

python scripts/run_studio_test.py --exe target/debug/ipa-tts.exe --test scripts/test_catalog_ui.py
"""
import argparse
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--test', type=Path, required=True)
    parser.add_argument('--cwd', type=Path, default=Path.cwd())
    parser.add_argument('--without-assets',action='store_true',help='Use nonexistent model/eSpeak paths for catalog isolation tests')
    args = parser.parse_args()
    with socket.socket() as reservation:
        reservation.bind(('127.0.0.1',0))
        port = reservation.getsockname()[1]
    environment = os.environ.copy()
    environment['IPA_TEST_URL'] = f'http://127.0.0.1:{port}'
    if args.without_assets:
        environment['KOKORO_MODEL_DIR']=str(Path(tempfile.gettempdir())/'ipa-deliberately-absent-models-v020')
        environment['ESPEAK_NG']=str(Path(tempfile.gettempdir())/'ipa-deliberately-absent-espeak-v020.exe')
        if Path(environment['KOKORO_MODEL_DIR']).exists() or Path(environment['ESPEAK_NG']).exists(): raise RuntimeError('Missing-assets fixture unexpectedly exists')
        environment['IPA_EXPECT_MISSING_MODELS']='1'
    with tempfile.TemporaryFile(mode='w+b') as log:
        server = subprocess.Popen([str(args.exe.resolve()),'serve',str(port)],cwd=args.cwd,env=environment,stdout=log,stderr=log,
                                  creationflags=subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0)
        try:
            for _ in range(100):
                if server.poll() is not None:
                    log.seek(0)
                    raise RuntimeError('Server exited: '+log.read().decode('utf-8',errors='replace'))
                try:
                    urllib.request.urlopen(environment['IPA_TEST_URL']+'/api/status',timeout=1).close()
                    break
                except OSError:
                    time.sleep(.1)
            else:
                raise RuntimeError('Owned server did not start')
            subprocess.run([sys.executable,str(args.test.resolve())],env=environment,check=True)
        finally:
            server.terminate()
            try: server.wait(timeout=10)
            except subprocess.TimeoutExpired: server.kill(); server.wait(timeout=10)

if __name__=='__main__': main()
