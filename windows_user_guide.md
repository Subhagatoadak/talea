# Talea Windows User Guide

This guide walks you through installing and running Talea on Windows.

## 1. Install prerequisites

1. **Rust** – Download and run the installer from [rustup.rs](https://rustup.rs). When asked, choose the *MSVC* toolchain.
2. **Visual Studio Build Tools** – Install the "Desktop development with C++" workload from [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/). This provides the linker that Rust uses on Windows.
3. **Python** – We recommend [Miniconda](https://docs.conda.io/en/latest/miniconda.html) or [Anaconda](https://www.anaconda.com/download). Talea expects Python 3.8+.
4. **Git** – If you do not already have Git, install it from [git-scm.com](https://git-scm.com/download/win).

## 2. Create a Python environment

Open the **Anaconda Prompt** (or PowerShell if using the Windows Store version of Python) and create an environment for Talea's NLP features:

```powershell
conda create --name talea-nlp python=3.11
conda activate talea-nlp

pip install -U spacy
python -m spacy download en_core_web_md
```

## 3. Link Python to Rust

Rust needs to know where the Python libraries live. From the project root, create `.cargo/config.toml` with the path to the `libs` directory inside your conda environment:

```toml
# .cargo/config.toml
[build]
rustflags = ["-L", "C:\\path\\to\\your\\conda\\env\\libs"]
```

You can find the exact path by running:

```powershell
python -c "import sys, os; print(os.path.join(sys.prefix, 'libs'))"
```

## 4. Build and run Talea

Clone the repository and launch the REPL:

```powershell
git clone https://github.com/your-username/talea.git
cd talea
conda activate talea-nlp
cargo run
```

If everything is set up correctly you will see the Talea prompt.

## 5. Run tests (optional)

To verify the build:

```powershell
conda activate talea-nlp
cargo test
```

## 6. Execute Talea scripts

Talea commands can be stored in files with the `.talea` extension. Run a script like this:

```powershell
cargo run -- examples\hello_world.talea
```

That’s it! You now have Talea running on Windows.
