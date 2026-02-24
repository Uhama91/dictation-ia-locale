#!/usr/bin/env bash
# build-whisper.sh — Compile whisper.cpp avec CoreML + Metal (Task 3)
#
# Usage:
#   ./scripts/build-whisper.sh
#
# Prérequis:
#   - macOS Apple Silicon (M1+)
#   - Xcode complet (pas seulement Command Line Tools) pour CoreML framework
#   - cmake >= 3.21
#   - python3 + coremltools (pour générer les modèles CoreML)
#
# Output:
#   vendor/whisper.cpp/build/src/libwhisper.a
#   vendor/whisper.cpp/ggml-metal.metal (copié dans resources/ via build.rs)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VENDOR_DIR="$PROJECT_ROOT/vendor"
WHISPER_DIR="$VENDOR_DIR/whisper.cpp"
BUILD_DIR="$WHISPER_DIR/build"
NCPU=$(sysctl -n hw.ncpu 2>/dev/null || echo 4)

echo "=== Build whisper.cpp avec CoreML + Metal ==="
echo "Project root: $PROJECT_ROOT"
echo "Whisper dir:  $WHISPER_DIR"
echo "CPUs:         $NCPU"

# Vérifier Xcode (nécessaire pour CoreML)
if ! xcode-select -p &>/dev/null || [ "$(xcode-select -p)" = "/Library/Developer/CommandLineTools" ]; then
    echo ""
    echo "AVERTISSEMENT: Xcode complet non détecté."
    echo "CoreML support nécessite Xcode.app (pas seulement Command Line Tools)."
    echo "Télécharger Xcode depuis l'App Store puis :"
    echo "  sudo xcode-select -s /Applications/Xcode.app/Contents/Developer"
    echo ""
    echo "Build sans CoreML (Metal uniquement)..."
    COREML_FLAG="-DWHISPER_COREML=OFF"
else
    COREML_FLAG="-DWHISPER_COREML=ON"
    echo "Xcode détecté: $(xcode-select -p)"
fi

# Cloner whisper.cpp si absent
if [ ! -d "$WHISPER_DIR" ]; then
    echo ""
    echo "Clonage de whisper.cpp..."
    git clone --depth 1 https://github.com/ggml-org/whisper.cpp.git "$WHISPER_DIR"
    echo "Clone terminé."
else
    echo "whisper.cpp déjà présent: $WHISPER_DIR"
    # Pull latest si souhaité (commenté pour build reproductible)
    # cd "$WHISPER_DIR" && git pull --depth 1
fi

# Configurer avec CMake
echo ""
echo "Configuration CMake..."
cmake -B "$BUILD_DIR" \
    -S "$WHISPER_DIR" \
    $COREML_FLAG \
    -DWHISPER_METAL=ON \
    -DBUILD_SHARED_LIBS=OFF \
    -DWHISPER_BUILD_TESTS=OFF \
    -DWHISPER_BUILD_EXAMPLES=OFF \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_OSX_ARCHITECTURES=arm64

# Compiler
echo ""
echo "Compilation (${NCPU} threads)..."
cmake --build "$BUILD_DIR" -j "$NCPU"

# Vérifier la lib
LIB_PATH="$BUILD_DIR/src/libwhisper.a"
GGML_LIB="$BUILD_DIR/ggml/src/libggml.a"

if [ -f "$LIB_PATH" ]; then
    echo ""
    echo "✓ libwhisper.a compilée : $LIB_PATH"
    ls -lh "$LIB_PATH"
else
    echo "ERREUR: $LIB_PATH non trouvée après compilation"
    exit 1
fi

# Générer les encodeurs CoreML (nécessite coremltools)
if [ "$COREML_FLAG" = "-DWHISPER_COREML=ON" ]; then
    echo ""
    echo "=== Génération des modèles CoreML ==="
    echo "Pour générer l'encodeur CoreML pour large-v3-turbo :"
    echo "  pip3 install coremltools openai-whisper"
    echo "  python3 $WHISPER_DIR/models/generate_coreml_model.py large-v3-turbo"
    echo ""
    echo "Le modèle .mlpackage sera dans :"
    echo "  $WHISPER_DIR/models/ggml-large-v3-turbo-encoder.mlpackage"
fi

echo ""
echo "=== Build terminé ==="
echo "Relancez cargo check pour vérifier l'intégration Rust."
