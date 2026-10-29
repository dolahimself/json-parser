from pathlib import Path
import os
import subprocess

ROOT = Path(__file__).resolve().parent
BUILD = ROOT / "target" / "checkpoints"
BUILD.mkdir(parents=True, exist_ok=True)
SUFFIX = ".exe" if os.name == "nt" else ""

for stage in sorted((ROOT / "checkpoints").iterdir()):
    if not stage.is_dir():
        continue
    executable = BUILD / (stage.name + SUFFIX)
    subprocess.run(["rustc", "--edition=2024", str(stage / "run.rs"), "-o", str(executable)], check=True)
    result = subprocess.run([str(executable)], capture_output=True, text=True, encoding="utf-8", check=True)
    expected = (stage / "results.txt").read_text(encoding="utf-8-sig")
    if result.stdout.strip() != expected.strip():
        raise RuntimeError(f"{stage.name}: output differs from results.txt")
    for test_source in [stage / "lesson_tests.rs", stage / "tests.rs"]:
        if test_source.exists():
            test_binary = BUILD / (stage.name + "-tests" + SUFFIX)
            subprocess.run(["rustc", "--edition=2024", "--test", str(test_source), "-o", str(test_binary)], check=True)
            subprocess.run([str(test_binary)], check=True)
    print(f"{stage.name}: passed", flush=True)