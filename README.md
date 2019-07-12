Substrate Node for Ladder network

Table of Contents
=================

   * [Hacking](#hacking)
      * [1. environment](#1-environment)
         * [1.1 Hacking on Substrate](#11-hacking-on-substrate)
            * [1.1.1 Linux and Mac](#111-linux-and-mac)
            * [1.1.2. Windows](#112-windows)
      * [2. Build](#2-build)
      * [3. UI](#3-ui)

# Hacking

## 1. environment

### 1.1 Hacking on Substrate

Ensure you have Rust and the support software installed:

#### 1.1.1 Linux and Mac

For Unix-based operating systems, you should run the following commands:

```
curl https://sh.rustup.rs -sSf | sh

rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup update stable
cargo install --git https://github.com/alexcrichton/wasm-gc
```

You will also need to install the following packages:

- Linux:

  ```
  sudo apt install cmake pkg-config libssl-dev git clang libclang-dev
  ```

- Mac:

  ```
  brew install cmake pkg-config openssl git llvm
  ```

To finish installation of Substrate, jump down to [Build](#2-build).

#### 1.1.2. Windows

If you are trying to set up Substrate on Windows, you should do the following:

1. First, you will need to download and install "Build Tools for Visual Studio:"

   - You can get it at this link: <https://aka.ms/buildtools>
   - Run the installation file: `vs_buildtools.exe`
   - Please ensure the Windows 10 SDK component is included when installing the Visual C++ Build Tools.
   - Restart your computer.

2. Next, you need to install Rust:

   - Detailed instructions are provided by the [Rust Book](https://doc.rust-lang.org/book/ch01-01-installation.html#installing-rustup-on-windows).
   - Download from: <https://www.rust-lang.org/tools/install>
   - Run the installation file: `rustup-init.exe` > Note that it should not prompt you to install vs_buildtools since you did it in step 1.
   - Choose "Default Installation."
   - To get started, you need Cargo’s bin directory (%USERPROFILE%\.cargo\bin) in your PATH environment variable. Future applications will automatically have the correct environment, but you may need to restart your current shell.

3. Then, you will need to run some commands in CMD to set up your Wasm Build Environment:

   ```
   rustup update nightly
   rustup update stable
   rustup target add wasm32-unknown-unknown --toolchain nightly
   ```

4. Next, you install wasm-gc, which is used to slim down Wasm files:

   ```
   cargo install --git https://github.com/alexcrichton/wasm-gc --force
   ```

5. Then, you need to install LLVM: <https://releases.llvm.org/download.html>

6. Next, you need to install OpenSSL, which we will do with `vcpkg`:

   ```
   mkdir \Tools
   cd \Tools
   git clone https://github.com/Microsoft/vcpkg.git
   cd vcpkg
   .\bootstrap-vcpkg.bat
   .\vcpkg.exe install openssl:x64-windows-static
   ```

7. After, you need to add OpenSSL to your System Variables:

   ```
   $env:OPENSSL_DIR = 'C:\Tools\vcpkg\installed\x64-windows-static'
   $env:OPENSSL_STATIC = 'Yes'
   [System.Environment]::SetEnvironmentVariable('OPENSSL_DIR', $env:OPENSSL_DIR, [System.EnvironmentVariableTarget]::User)
   [System.Environment]::SetEnvironmentVariable('OPENSSL_STATIC', $env:OPENSSL_STATIC, [System.EnvironmentVariableTarget]::User)
   ```

8. Finally, you need to install `cmake`: <https://cmake.org/download/>

## 2. Build

Then, grab the source code:

```bash
git clone https://github.com/laddernetwork/ladder.git
cd ladder
```

Then build the code:

```bash
./build.sh

cargo build
```

You can start a development chain with:
```bash
cargo run -- --dev
```

## 3. UI
* visit https://polkadot.js.org/apps/ .
* Settings => Local Node.
* Settings => Developer => Manually enter your custom type definitions as valid JSON
```
{
  "Symbol": "u64",
  "OrderPair": {
    "share": "Symbol",
    "money": "Symbol"
  },
  "Status": {
    "_enum": [
      "Lcoking",
      "Unlock",
      "Withdraw"
    ]
  },
  "OtcStatus": {
    "_enum": [
      "New",
      "Half",
      "Done"
    ]
  },
  "TokenType": {
    "_enum": [
      "Free",
      "OTC",
      "WithDraw",
      "Reward"
    ]
  },
  "OrderT": {
    "pair": "OrderPair",
    "index": "u64",
    "who": "AccountId",
    "amount": "Symbol",
    "price": "Symbol",
    "already_deal": "Symbol",
    "status": "OtcStatus",
    "longindex": "u128",
    "reserved": "bool",
    "acc": "Vec<u8>"
  },
  "TokenInfoT": {
    "id": "u64",
    "sender": "Vec<u8>",
    "beneficiary": "AccountId",
    "value": "Balance",
    "cycle": "u64",
    "reward": "Balance",
    "txhash": "Vec<u8>",
    "status": "Status",
    "now_cycle": "u64",
    "reserved": "bool"
  }
}
```
