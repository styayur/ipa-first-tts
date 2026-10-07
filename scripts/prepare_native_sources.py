# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Stya Yur Open Source Studio
"""Fetch pinned native corresponding sources and preserve upstream notices.

Downloads are a release preparation step, never part of runtime synthesis.
Run with Python 3.11+. Sources remain unmodified compressed snapshots.
"""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import tarfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
CACHE = ROOT / ".cache/upstream"
NOTICES = ROOT / "licenses/upstream"


def records(path):
    if path.suffix == ".zip":
        with zipfile.ZipFile(path) as archive:
            for name in archive.namelist():
                if not name.endswith("/"):
                    yield name, lambda name=name: archive.read(name)
    else:
        with tarfile.open(path, "r:*") as archive:
            for member in archive:
                if member.isfile():
                    yield member.name, lambda member=member: archive.extractfile(member).read()


def fetch(name, url, expected=None):
    CACHE.mkdir(parents=True, exist_ok=True)
    file = CACHE / name
    if not file.is_file():
        part = file.with_name(file.name + ".partial")
        subprocess.run(["curl.exe" if __import__('os').name == 'nt' else 'curl', "-sS", "-L", "--fail", "--retry", "2", url, "-o", str(part)], check=True)
        part.replace(file)
    digest = hashlib.file_digest(file.open("rb"), "sha256").hexdigest()
    if expected and digest.lower() != expected.lower():
        raise RuntimeError(f"Native source hash mismatch: {name}")
    return {"file": name, "url": url, "sha256": digest}


def main():
    components = [
        ("sherpa-onnx", "sherpa-onnx-v1.13.8.tar.gz", "https://codeload.github.com/k2-fsa/sherpa-onnx/tar.gz/refs/tags/v1.13.8", None),
        ("espeak-ng-cli", "espeak-ng-1.52.0.tar.gz", "https://codeload.github.com/espeak-ng/espeak-ng/tar.gz/refs/tags/1.52.0", None),
        ("onnxruntime", "onnxruntime-v1.28.2.tar.gz", "https://codeload.github.com/microsoft/onnxruntime/tar.gz/refs/tags/v1.28.2", None),
    ]
    # Exact source URLs/hashes come from the locked sherpa tag's build scripts.
    sherpa = CACHE / "sherpa-onnx-v1.13.8.tar.gz"
    fetch(*components[0][1:])
    required = {"espeak-ng-for-piper.cmake", "piper-phonemize.cmake", "kaldi-decoder.cmake", "kaldi-native-fbank.cmake", "simple-sentencepiece.cmake", "json.cmake", "hclust-cpp.cmake", "websocketpp.cmake", "asio.cmake"}
    for name, read in records(sherpa):
        if Path(name).name not in required or "/cmake/" not in name: continue
        text = read().decode("utf-8")
        match = re.search(r'set\([^\s]+_URL\s+"(https://github.com/[^"\s]+)"', text)
        if not match: raise RuntimeError(f"Missing pinned URL in {name}")
        url = match.group(1)
        filename = url.rsplit("/", 1)[-1]
        owner, repo = url.split("/")[3:5]
        prefix = repo + "-"
        if not filename.startswith(prefix): filename = prefix + filename
        expected = re.search(r'set\([^\s]+_HASH\s+"SHA256=([a-fA-F0-9]+)"', text)
        components.append((repo, filename, url, expected.group(1) if expected else None))
    results = []
    urls = set()
    # Fetch recursively pinned build dependencies for decoder/fbank (OpenFst,
    # kaldifst and KissFFT). Ignore unrelated test/docs/platform downloads.
    index = 0
    while index < len(components):
        component, filename, url, expected = components[index]; index += 1
        if url in urls: continue
        urls.add(url)
        print(f"Preparing source: {component} / {filename}", flush=True)
        result = fetch(filename, url, expected); result["component"] = component
        results.append(result)
        for name, read in records(CACHE / filename):
            basename = Path(name).name
            # Read only notices and small build files, not binary/model payloads.
            notice = basename.upper().startswith(("LICENSE", "COPYING", "NOTICE", "AUTHORS", "THIRDPARTYNOTICES", "COPYRIGHT"))
            if notice:
                destination = NOTICES / component / Path(name).relative_to(Path(name).parts[0])
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(read())
            if component in {"kaldi-decoder", "kaldi-native-fbank", "kaldifst"} and (name.endswith(".cmake") or basename == "CMakeLists.txt"):
                text = read().decode("utf-8", "replace")
                for match in re.finditer(r'set\([^\s]+_URL\s+"(https://github.com/[^"\s]+)"', text):
                    dep_url = match.group(1)
                    if "$" in dep_url or not any(s in dep_url for s in ["/kaldifst/", "/openfst/", "/kissfft/"]): continue
                    dep_repo = dep_url.split("/")[4]
                    dep_name = dep_url.rsplit("/", 1)[-1]
                    if not dep_name.startswith(dep_repo + "-"): dep_name = dep_repo + "-" + dep_name
                    hash_match = re.search(r'set\([^\s]+_HASH\s+"SHA256=([a-fA-F0-9]+)"', text)
                    components.append((dep_repo, dep_name, dep_url, hash_match.group(1) if hash_match else None))
    (ROOT / "licenses/native-sources.json").write_text(json.dumps(results, ensure_ascii=False, indent=2)+"\n", encoding="utf-8")
    print(f"Preserved {len(results)} native source archives and their notices", flush=True)


if __name__ == "__main__":
    main()
