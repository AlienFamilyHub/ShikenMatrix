# ShikenMatrix macOS UI

macOS SwiftUI 应用用于配置和控制 ShikenMatrix Reporter。

## 构建步骤

### 1. 构建 Rust 库

```bash
cd macos-ui
./build-rust.sh
```

这会将编译好的 `libshikenmatrix.dylib` 放到 `rust-lib/` 目录。

### 2. 在 Xcode 中配置项目

#### 添加 Rust 动态库

1. 打开 `ShikenMatrix.xcodeproj`
2. 选择项目 → 选择 ShikenMatrix target
3. 进入 "Build Phases" → "Link Binary With Libraries"
4. 点击 "+" 添加 `rust-lib/libshikenmatrix.dylib`
5. 确保 "Embed Frameworks" 中也添加了这个库

#### 设置搜索路径

在 "Build Settings" 中：

**Library Search Paths:**
```
$(PROJECT_DIR)/rust-lib
```

**Header Search Paths:**
```
$(PROJECT_DIR)/rust-lib
```

### 3. 构建并运行

在 Xcode 中点击 ▶️ 运行按钮。

## 项目结构

```
ShikenMatrix/
├── ShikenMatrixApp.swift    # App 入口
├── ContentView.swift         # 主界面（设置和状态）
├── RustBridge.swift          # Rust FFI 桥接
└── Assets.xcassets/          # 资源文件

rust-lib/
├── libshikenmatrix.dylib     # Rust 动态库（运行 build-rust.sh 生成）
└── shikenmatrix.h            # C 头文件
```

## 功能

- ✅ 加载/保存配置
- ✅ 启动/停止 Reporter
- ✅ 实时状态显示
- ✅ 表单验证

## 开发注意事项

- 修改 Rust 代码后需要重新运行 `build-rust.sh`
- Rust 库是 ARM64 架构，在 Apple Silicon Mac 上运行
- 配置文件保存在 `~/.shikenmatrix/config.toml`
