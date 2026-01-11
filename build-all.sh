#!/bin/bash
# 一键构建脚本：Rust 库 + macOS SwiftUI 应用

set -e  # 遇到错误立即退出

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MACOS_UI_DIR="$SCRIPT_DIR/macos-ui"
XCODE_PROJECT="$MACOS_UI_DIR/ShikenMatrix/ShikenMatrix.xcodeproj"
SCHEME="ShikenMatrix"

# 解析命令行参数
BUILD_MODE="Release"
BUILD_CONFIG="release"
CLEAN_BUILD=""

print_usage() {
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  --debug       使用 Debug 模式构建"
    echo "  --clean       清理构建缓存"
    echo "  -h, --help    显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0                # Release 模式构建"
    echo "  $0 --debug        # Debug 模式构建"
    echo "  $0 --debug --clean # Debug 模式清理后构建"
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_MODE="Debug"
            BUILD_CONFIG="debug"
            shift
            ;;
        --clean)
            CLEAN_BUILD="--clean"
            shift
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            echo -e "${RED}未知选项: $1${NC}"
            print_usage
            exit 1
            ;;
    esac
done

# 打印构建信息
echo -e "${CYAN}╔══════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║    ShikenMatrix 一键构建脚本            ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}构建模式:${NC} ${YELLOW}$BUILD_MODE${NC}"
echo ""

# ==================== Step 1: 构建 Rust 库 ====================
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Step 1: 构建 Rust 库${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

cd "$SCRIPT_DIR"

if [ "$BUILD_CONFIG" = "debug" ]; then
    echo -e "${YELLOW}编译 Rust 库 (Debug)...${NC}"
    cargo build
    TARGET_DIR="target/debug"
else
    echo -e "${YELLOW}编译 Rust 库 (Release)...${NC}"
    cargo build --release
    TARGET_DIR="target/release"
fi

if [ ! -f "$TARGET_DIR/libshikenmatrix.dylib" ]; then
    echo -e "${RED}❌ Rust 库构建失败${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Rust 库编译成功${NC}"
echo ""

# ==================== Step 2: 生成 C 头文件 ====================
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Step 2: 生成 C 头文件${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# cbindgen 需要 nightly，如果失败就使用已有的头文件或跳过
if cbindgen --config cbindgen.toml --crate shikenmatrix --output shikenmatrix.h 2>/dev/null; then
    echo -e "${GREEN}✓ 生成 shikenmatrix.h${NC}"
else
    echo -e "${YELLOW}⚠ cbindgen 需要 nightly Rust，跳过头文件生成${NC}"
    if [ -f "shikenmatrix.h" ]; then
        echo -e "${YELLOW}  使用现有的 shikenmatrix.h${NC}"
    fi
fi
echo ""

# ==================== Step 3: 复制和签名库文件 ====================
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Step 3: 复制和签名库文件${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

RUST_LIB_DIR="$MACOS_UI_DIR/rust-lib"
mkdir -p "$RUST_LIB_DIR"

cp "$TARGET_DIR/libshikenmatrix.dylib" "$RUST_LIB_DIR/"
if [ -f "shikenmatrix.h" ]; then
    cp "shikenmatrix.h" "$RUST_LIB_DIR/"
fi

# 修复库的安装路径
install_name_tool -id "@rpath/libshikenmatrix.dylib" "$RUST_LIB_DIR/libshikenmatrix.dylib" 2>/dev/null || true

# Release 模式优化
if [ "$BUILD_CONFIG" = "release" ]; then
    strip -x "$RUST_LIB_DIR/libshikenmatrix.dylib" 2>/dev/null || true
fi

# 代码签名
echo "代码签名..."
if codesign --force --sign "LG25FB2235" "$RUST_LIB_DIR/libshikenmatrix.dylib" 2>/dev/null; then
    echo -e "${GREEN}✓ 使用 Team ID LG25FB2235 签名${NC}"
elif codesign --force --sign "Apple Development: tianxiang_tnxg@outlook.com" "$RUST_LIB_DIR/libshikenmatrix.dylib" 2>/dev/null; then
    echo -e "${GREEN}✓ 使用 Apple Development 证书签名${NC}"
else
    echo -e "${YELLOW}使用 ad-hoc 签名（开发模式）${NC}"
    codesign --force --sign - "$RUST_LIB_DIR/libshikenmatrix.dylib" 2>/dev/null || true
fi

echo -e "${GREEN}✓ 库文件准备完成${NC}"
echo ""

# ==================== Step 4: 构建 Xcode 项目 ====================
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Step 4: 构建 macOS 应用${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [ ! -d "$XCODE_PROJECT" ]; then
    echo -e "${RED}❌ 找不到 Xcode 项目: $XCODE_PROJECT${NC}"
    exit 1
fi

cd "$MACOS_UI_DIR"

# 构建 Xcode 项目
if [ -n "$CLEAN_BUILD" ]; then
    echo -e "${YELLOW}清理构建缓存...${NC}"
    xcodebuild clean -project "$XCODE_PROJECT" -scheme "$SCHEME" > /dev/null || true
    echo ""
fi

echo -e "${YELLOW}使用 Xcode 构建 $BUILD_MODE 模式...${NC}"

# 使用 xcodebuild 构建，匹配 Xcode 的开发者设置
xcodebuild build \
    -project "$XCODE_PROJECT" \
    -scheme "$SCHEME" \
    -configuration "$BUILD_MODE" \
    -derivedDataPath "$MACOS_UI_DIR/build" \
    CODE_SIGN_IDENTITY="Apple Development" \
    CODE_SIGN_STYLE="Automatic" \
    DEVELOPMENT_TEAM="" \
    | grep -E "(error|warning|BUILD SUCCEEDED|BUILD FAILED)" || true

BUILD_STATUS=${PIPESTATUS[0]}

if [ $BUILD_STATUS -eq 0 ]; then
    echo ""
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✅ 构建成功！${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${CYAN}应用位置:${NC}"
    APP_PATH="$MACOS_UI_DIR/build/Build/Products/$BUILD_MODE/ShikenMatrix.app"
    if [ -d "$APP_PATH" ]; then
        echo -e "  ${GREEN}$APP_PATH${NC}"
        echo ""
        echo -e "${CYAN}运行应用:${NC}"
        echo -e "  ${YELLOW}open \"$APP_PATH\"${NC}"
    else
        echo -e "  ${YELLOW}在 Xcode 中按 Cmd+R 运行${NC}"
    fi
    echo ""
else
    echo ""
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}❌ 构建失败${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${YELLOW}提示: 查看上方的错误信息，或在 Xcode 中构建以查看详细日志${NC}"
    echo ""
    exit 1
fi
