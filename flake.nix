# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CREBAIN - Adaptive Response & Awareness System (ARAS)
# Adaptives Reaktions- und Aufklärungssystem
#
# Cross-platform Nix flake for macOS (Apple Silicon) and Linux
#
# Usage:
#   nix develop                    # Enter dev shell (auto-detects platform)
#   nix develop .#cuda             # Force CUDA dev shell
#   nix develop .#cpu-only         # CPU-only dev shell
#   nix build                      # Build for current platform
#   nix run                        # Run the application
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{
  description = "CREBAIN - Adaptive Response & Awareness System (ARAS)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    zig-overlay = {
      url = "github:mitchellh/zig-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, zig-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          rust-overlay.overlays.default
          zig-overlay.overlays.default
        ];
        isLinuxSystem = builtins.match ".*-linux" system != null;
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
            cudaSupport = isLinuxSystem;
          };
        };

        isDarwin = pkgs.stdenv.isDarwin;
        isLinux = pkgs.stdenv.isLinux;
        isAarch64 = pkgs.stdenv.hostPlatform.isAarch64;

        hasCuda = isLinux && (
          builtins.pathExists "/dev/nvidia0"
          || builtins.pathExists "/proc/driver/nvidia/version"
          || builtins.pathExists "/run/current-system/sw/bin/nvidia-smi"
          || builtins.pathExists "/run/wrappers/bin/nvidia-smi"
          || builtins.pathExists "/usr/bin/nvidia-smi"
          || builtins.pathExists "/bin/nvidia-smi"
        );

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = if isDarwin
            then [ "aarch64-apple-darwin" "x86_64-apple-darwin" ]
            else [ "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu" ];
        };

        commonBuildInputs = with pkgs; [
          rustToolchain
          cargo
          cargo-watch
          cargo-edit
          zig-overlay.packages.${system}.master
          pkg-config
          cmake
          gnumake
          openssl
          zlib
        ];

        darwinBuildInputs = with pkgs; [
          darwin.apple_sdk.frameworks.CoreML
          darwin.apple_sdk.frameworks.Metal
          darwin.apple_sdk.frameworks.MetalPerformanceShaders
          darwin.apple_sdk.frameworks.Accelerate
          darwin.apple_sdk.frameworks.Vision
          darwin.apple_sdk.frameworks.CoreVideo
          darwin.apple_sdk.frameworks.CoreMedia
          darwin.apple_sdk.frameworks.CoreGraphics
          darwin.apple_sdk.frameworks.AVFoundation
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
          darwin.apple_sdk.frameworks.Foundation
          darwin.apple_sdk.frameworks.IOKit
          libiconv
          darwin.libobjc
        ];

        linuxBuildInputs = with pkgs; [
          onnxruntime
          alsa-lib
          libappindicator
          vulkan-loader
          vulkan-headers
          vulkan-tools
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libxcb
          dbus
        ];

        onnxruntimeMerged = pkgs.symlinkJoin {
          name = "onnxruntime-merged";
          paths = [ pkgs.onnxruntime ] ++ pkgs.lib.optional (pkgs.onnxruntime ? dev) pkgs.onnxruntime.dev;
        };

        cudaBuildInputs = with pkgs; [
          cudaPackages.cudatoolkit
          cudaPackages.cudnn
          cudaPackages.tensorrt
          cudaPackages.nccl
        ];

        platformBuildInputs =
          if isDarwin then darwinBuildInputs
          else linuxBuildInputs ++ (if hasCuda then cudaBuildInputs else []);

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = commonBuildInputs ++ platformBuildInputs;

          RUST_BACKTRACE = "1";
          RUST_LOG = "info";
          ORT_SKIP_DOWNLOAD = "1";
          CARGO_FEATURES = if isDarwin then "" else (if hasCuda then "cuda" else "");

          LD_LIBRARY_PATH = if isLinux then
            pkgs.lib.makeLibraryPath (linuxBuildInputs ++ (if hasCuda then cudaBuildInputs else []))
          else "";

          CUDA_PATH = if hasCuda then "${pkgs.cudaPackages.cudatoolkit}" else "";
          TENSORRT_ROOT = if hasCuda then "${pkgs.cudaPackages.tensorrt}" else "";
          ORT_DYLIB_PATH = if isLinux then "${onnxruntimeMerged}/lib/libonnxruntime.so" else "";

          shellHook = ''
            echo ""
            echo "╔══════════════════════════════════════════════════════════════════╗"
            echo "║        CREBAIN - Adaptive Response & Awareness System            ║"
            echo "║        Adaptives Reaktions- und Aufklärungssystem (ARAS)         ║"
            echo "╠══════════════════════════════════════════════════════════════════╣"
            ${if isDarwin then ''
            echo "║  Platform:   macOS ${if isAarch64 then "(Apple Silicon)" else "(Intel)"}                              ║"
            echo "║  ML Backend: CoreML / MLX                                        ║"
            echo "║  GPU:        Metal                                               ║"
            '' else ''
            echo "║  Platform:   Linux (NixOS)                                       ║"
            ${if hasCuda then ''
            echo "║  ML Backend: CUDA / TensorRT                                     ║"
            echo "║  Features:   --features cuda                                     ║"
            '' else ''
            echo "║  ML Backend: ONNX Runtime (CPU)                                  ║"
            echo "║  Features:   (none)                                              ║"
            ''}
            ''}
            echo "╠══════════════════════════════════════════════════════════════════╣"
            echo "║  Rust:   $(rustc --version | cut -d' ' -f2)                                                ║"
            echo "║  Zig:    $(zig version 2>/dev/null || echo "not found")                                                ║"
            echo "╠══════════════════════════════════════════════════════════════════╣"
            echo "║  Commands:                                                       ║"
            echo "║    cargo run              - Build and run the app                 ║"
            echo "║    cargo run --release    - Release build and run                 ║"
            echo "║    cargo test --workspace - Run all tests                         ║"
            echo "║    cargo clippy           - Lint                                 ║"
            echo "╚══════════════════════════════════════════════════════════════════╝"
            echo ""

            ${if isDarwin then ''
            export CREBAIN_BACKEND=coreml
            export CREBAIN_ZENOH=1
            '' else ''
            ${if hasCuda then ''
            export CREBAIN_BACKEND=tensorrt
            export CREBAIN_CUDA=1
            '' else ''
            export CREBAIN_BACKEND=onnx
            export CREBAIN_CPU_ONLY=1
            ''}
            export CREBAIN_ZENOH=1
            export RMW_IMPLEMENTATION=rmw_zenoh_cpp
            ''}

            if [ -d /run/opengl-driver/lib ]; then
              export LD_LIBRARY_PATH="/run/opengl-driver/lib:$LD_LIBRARY_PATH"
            fi
            if [ -d /run/opengl-driver-32/lib ]; then
              export LD_LIBRARY_PATH="/run/opengl-driver-32/lib:$LD_LIBRARY_PATH"
            fi
          '';
        };

        devShells.cuda = pkgs.mkShell {
          buildInputs = commonBuildInputs ++ linuxBuildInputs ++ cudaBuildInputs;
          RUST_BACKTRACE = "1";
          ORT_SKIP_DOWNLOAD = "1";
          CARGO_FEATURES = "cuda";
          CUDA_PATH = "${pkgs.cudaPackages.cudatoolkit}";
          TENSORRT_ROOT = "${pkgs.cudaPackages.tensorrt}";
          ORT_DYLIB_PATH = "${onnxruntimeMerged}/lib/libonnxruntime.so";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (linuxBuildInputs ++ cudaBuildInputs);

          shellHook = ''
            echo "CREBAIN Development Environment (CUDA Forced)"
            export CREBAIN_BACKEND=tensorrt
            export CREBAIN_CUDA=1
            export CREBAIN_ZENOH=1
            if [ -d /run/opengl-driver/lib ]; then
              export LD_LIBRARY_PATH="/run/opengl-driver/lib:$LD_LIBRARY_PATH"
            fi
            if [ -d /run/opengl-driver-32/lib ]; then
              export LD_LIBRARY_PATH="/run/opengl-driver-32/lib:$LD_LIBRARY_PATH"
            fi
          '';
        };

        devShells.cpu-only = pkgs.mkShell {
          buildInputs = commonBuildInputs ++ (if isLinux then linuxBuildInputs else darwinBuildInputs);
          RUST_BACKTRACE = "1";
          ORT_SKIP_DOWNLOAD = "1";
          CARGO_FEATURES = "";
          ORT_DYLIB_PATH = if isLinux then "${onnxruntimeMerged}/lib/libonnxruntime.so" else "";

          shellHook = ''
            echo "CREBAIN Development Environment (CPU Only)"
            export CREBAIN_BACKEND=onnx
            export CREBAIN_CPU_ONLY=1
            export CREBAIN_ZENOH=1
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "crebain";
          version = "0.4.0";

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
            cmake
            rustToolchain
            makeWrapper
          ];

          buildInputs = commonBuildInputs ++ (
            if pkgs.stdenv.isLinux then linuxBuildInputs ++ (if hasCuda then cudaBuildInputs else []) else []
          );
          ORT_SKIP_DOWNLOAD = "1";

          preBuild = ''
            echo "Building Zig detector..."
            cd native/zig-detector
            ${zig-overlay.packages.${system}.master}/bin/zig build -Doptimize=ReleaseFast ${
              if hasCuda then "-Dcuda=true -Dcuda-path=${pkgs.cudaPackages.cudatoolkit} -Donnx=true -Donnx-path=${onnxruntimeMerged}" else ""
            }
            cd ../..
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/crebain-app $out/bin/crebain

            ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
            wrapProgram $out/bin/crebain \
              --set-default ORT_DYLIB_PATH "${onnxruntimeMerged}/lib/libonnxruntime.so" \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath (linuxBuildInputs ++ (if hasCuda then cudaBuildInputs else []))}" \
              --prefix LD_LIBRARY_PATH : "/run/opengl-driver/lib" \
              --prefix LD_LIBRARY_PATH : "/run/opengl-driver-32/lib"
            ''}

            mkdir -p $out/share/crebain/models
            cp -r resources/* $out/share/crebain/models/ 2>/dev/null || true

            mkdir -p $out/lib
            cp native/zig-detector/zig-out/lib/libcrebain_detector.* $out/lib/ 2>/dev/null || true
          '';

          meta = with pkgs.lib; {
            description = "CREBAIN - Adaptive Response & Awareness System (ARAS)";
            homepage = "https://github.com/crebain/crebain";
            license = licenses.mit;
            platforms = platforms.linux ++ platforms.darwin;
          };
        };

        packages.docker = pkgs.dockerTools.buildImage {
          name = "crebain";
          tag = "latest";

          copyToRoot = pkgs.buildEnv {
            name = "crebain-root";
            paths = [ self.packages.${system}.default ];
          };

          config = {
            Cmd = [ "/bin/crebain" ];
            Env = [
              "RUST_LOG=info"
              "CREBAIN_MODEL_PATH=/share/crebain/models/yolov8s.onnx"
            ];
          };
        };

        packages.zig-detector = pkgs.stdenv.mkDerivation {
          pname = "crebain-zig-detector";
          version = "0.1.0";

          src = ./native/zig-detector;

          nativeBuildInputs = [ zig-overlay.packages.${system}.master ];

          buildInputs = if hasCuda then [
            pkgs.cudaPackages.cudatoolkit
            pkgs.cudaPackages.cudnn
            onnxruntimeMerged
          ] else [];

          buildPhase = ''
            zig build -Doptimize=ReleaseFast ${
              if hasCuda then "-Dcuda=true -Donnx=true -Donnx-path=${onnxruntimeMerged}" else ""
            }
          '';

          installPhase = ''
            mkdir -p $out/lib $out/include
            cp zig-out/lib/* $out/lib/
            cp src/crebain_detector.h $out/include/
          '';
        };
      }
    );
}