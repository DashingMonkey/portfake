# Portfake

一款轻量级的 API 模拟服务器，基于 Tauri + Vue 3 + Rust 构建。

## 功能特性

- **Mock API 管理**：创建并组织 Mock API 接口到集合中
- **本地模拟服务器**：运行本地服务器以提供 Mock API 服务
- **请求编辑器**：为每个接口定义方法、URL、请求头和响应体
- **响应模板**：使用模板语法生成动态响应
- **集合管理**：将 Mock 接口组织到文件夹中，支持拖拽排序
- **导入/导出**：轻松共享集合
- **类 VS Code 界面**：浅色/深色主题、可调整大小的面板、标签页

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 + TypeScript |
| 样式 | Tailwind CSS |
| 状态管理 | Pinia |
| HTTP 服务器 | Axum (Rust) |
| 数据库 | SQLite (rusqlite) |

## 目录结构

```
portfake/
├── src/                          # Vue 前端
│   ├── components/               # UI 组件
│   │   ├── common/              # EmptyState、SettingsDialog、Toast、MethodSelect
│   │   ├── layout/              # AppHeader
│   │   ├── tabs/                # TabBar
│   │   └── workspace/           # SidebarPanel、MainWorkspace、RequestEditor
│   ├── stores/                  # Pinia 状态管理
│   │   ├── tabs.ts             # 标签页管理
│   │   ├── settings.ts         # 主题、UI 状态
│   │   ├── server.ts           # 模拟服务器状态
│   │   ├── collections.ts      # 集合树管理
│   │   └── types.ts            # TypeScript 类型定义
│   └── composables/              # Vue 组合式函数
│       └── useToast.ts          # Toast 通知
│
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── commands/            # Tauri 命令处理
│   │   │   ├── server.rs       # 模拟服务器控制（启动/停止）
│   │   │   ├── collections.rs  # 集合 CRUD
│   │   │   ├── requests.rs     # 请求 CRUD
│   │   │   └── examples.rs     # 示例生成
│   │   ├── db/                  # 数据库层
│   │   │   ├── mod.rs          # SQLite 连接
│   │   │   ├── schema.rs      # 表结构定义
│   │   │   ├── collections.rs # 集合操作
│   │   │   ├── requests.rs    # 请求操作
│   │   │   └── examples.rs    # 示例操作
│   │   ├── models/              # 数据模型
│   │   ├── server/              # Axum HTTP 服务器
│   │   │   └── mod.rs          # 模拟服务器实现
│   │   ├── template/            # 响应模板引擎
│   │   │   └── mod.rs          # 模板解析与渲染
│   │   └── example_gen/         # 示例响应生成器
│   │       └── mod.rs
│   └── Cargo.toml
│
├── package.json                  # Node 依赖
├── vite.config.ts               # Vite 打包配置
├── tailwind.config.js           # Tailwind 主题配置
└── tauri.conf.json              # Tauri 应用配置
```

## 数据存储

### SQLite 数据库（按工作区）

每个工作区是位于可执行文件目录下的独立 `.db` 文件。

| 表名 | 说明 |
|------|------|
| `collections` | 文件夹层级结构（id、parent_id、name、sort_order） |
| `requests` | Mock 接口（method、path、headers、body、response_delay） |
| `examples` | 关联到请求的示例响应 |

### 浏览器 localStorage（按工作区）

| 键名 | 说明 |
|------|------|
| `portfake-{workspace}-tabs` | 打开的标签页状态 |
| `portfake-{workspace}-drafts` | 未保存的请求更改 |
| `portfake-settings` | 全局设置（主题、当前工作区、UI 状态） |

## 依赖列表

### 前端（package.json）

```
vue@^3.5.32
pinia@^3.0.4
@headlessui/vue
@tauri-apps/api@^2.0.0
@tauri-apps/cli@^2.0.0
@vitejs/plugin-vue@^5.0.0
tailwindcss
typescript
vite
vue-tsc
```

### 后端（Cargo.toml）

```
tauri@2
tauri-plugin-shell
serde/serde_json
tokio (full)
rusqlite (bundled)
uuid
chrono
axum
tower
tower-http (cors)
log/env_logger
anyhow
rand
regex
```

## 构建

```bash
# 安装前端依赖
npm install

# 开发模式运行
npm run tauri dev

# 生产环境构建
npm run tauri build
```

## 模拟服务器

Portfake 运行一个基于 Axum 的本地 HTTP 服务器来提供 Mock API：

- 默认地址：`http://localhost:3210`
- 提供所有配置的 Mock 接口
- 支持动态响应模板
- 可配置响应延迟用于测试

## 窗口与界面

- 默认尺寸：1280x800，最小：900x600
- 自定义无边框窗口，带标题栏控制按钮
- 类 VS Code 活动栏和侧边栏布局
- 浅色/深色主题支持
