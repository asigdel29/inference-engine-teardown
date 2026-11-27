"""Invoke the native runner through the original Python module entry point."""
import os
from pathlib import Path
import shutil
import sys
import sysconfig


def main():
    """Forward all arguments to the wheel's binary or a source-build override."""
    command = "inference-engine-teardown"
    binary = os.environ.get("INFERENCE_TEARDOWN_BIN")
    if not binary:
        name = command + (".exe" if os.name == "nt" else "")
        installed = Path(sysconfig.get_path("scripts")) / name
        binary = str(installed) if installed.is_file() else shutil.which(command)
    if not binary:
        sys.exit("Install this package or set INFERENCE_TEARDOWN_BIN to the built executable.")
    os.execv(binary, [binary, *sys.argv[1:]])


if __name__ == "__main__":
    main()
