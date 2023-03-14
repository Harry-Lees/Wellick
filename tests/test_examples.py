import platform
import subprocess
from pathlib import Path

import pytest


def run_example(file: str) -> tuple[bytes, bytes, int]:
    """
    Run an example and check that it compiles.

    Returns a tuple of stdout, stderr, and the return code.
    """

    root = Path(__file__).parent.parent
    file_path = root / "examples" / file

    compiler_output = subprocess.run(["cargo", "run", "--release", str(file_path)])
    if compiler_output.returncode != 0:
        raise RuntimeError("Failed to compile")

    if platform.system() == "Windows":
        subprocess.check_call(["LINK", "a.out", "builtins", "/MD", "/Wall"])
        result = subprocess.run(["a.exe"])
        return result.stdout, result.stderr, result.returncode

    raise NotImplementedError(
        f"Unsuported platform, expected Windows, found {platform.system()}"
    )


def test_builtins() -> None:
    """Test that the builtins example compiles."""
    file = "builtins.wellick"
    stdout, stderr, retcode = run_example(file)
    assert retcode == 20


def test_default_const() -> None:
    """Test that the default_const example compiles."""
    file = "default_const.wellick"
    with pytest.raises(RuntimeError):
        stdout, stderr, retcode = run_example(file)
