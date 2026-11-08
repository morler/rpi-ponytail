# rpi-ponytail

把 [Ponytail](https://github.com/DietrichGebert/ponytail) 的"最小完整改动"规则带到 rpi。

Ponytail 本身只是一段提示词：先读懂任务和它触及的代码，再用最少的新代码解决整个问题；
无法删除的地方不删，但信任边界校验、防数据丢失的错误处理、安全和可访问性永不裁掉。

本仓库把它做成 rpi 原生插件（Rust `cdylib`）：

- **自动生效**：每轮请求前，把当前级别的原版规则追加到系统提示词，无需手动调用。
- **可切换级别**：`/ponytail lite|full|ultra|off`，切换结果写入配置文件，重启后保持。
- **附带专项技能**：review / audit / debt / gain / help，模型可按需读取或手动触发。

不访问网络，不执行外部命令，不收集任何数据。

## 安装

需要已构建好的 rpi（`rpi --version` 可用）与 Rust 工具链（`rpi package link` 会调用 Cargo）。

在**目标项目**目录执行（把路径换成本仓库的实际位置）：

```sh
rpi package link --path /path/to/rpi-ponytail
mkdir -p .rpi/skills
cp -R /path/to/rpi-ponytail/skills/ponytail-* .rpi/skills/
```

- 第 1 步把插件编译并链接到 `.rpi/packages/rpi-ponytail/`，之后在该项目里启动 `rpi` 会自动加载。
  想用 debug 产物或锁定依赖版本，可加 `--debug` / `--locked`；已链接过则加 `--force`。
- 第 2 步复制专项技能。**`rpi package link` 只链接动态库，不会复制 `skills/`**，所以这一步必须手动做。
  更新本仓库后（例如改了技能文案）需要重新复制一次。

验证加载：在项目里运行 `rpi` 后输入 `/ponytail status`，或非交互检查：

```sh
printf '{"id":"1","type":"get_commands"}\n' | rpi --mode rpc --offline --approve --package-only \
  | grep -o '"name":"ponytail"'
```

### 卸载

```sh
rpi package unlink rpi-ponytail
rm -rf .rpi/skills/ponytail-{review,audit,debt,gain,help}
```

卸载不会删除用户的 Ponytail 配置文件（`~/.config/ponytail/config.json` 等），需要时自行删除。

## 命令

| 命令 | 行为 |
| --- | --- |
| `/ponytail lite\|full\|ultra` | 切换级别并保存，`/ponytail off` 停止自动注入 |
| `/ponytail status` | 显示当前级别与配置的默认级别 |
| `/ponytail`（无参数） | 已启用时等同 `status`；已关闭时按默认级别重新启用（默认值为 `off` 时启用 `full`） |
| `/skill:ponytail-review` | 按资深审查者视角审当前改动：bug、安全、真实负载、缺测试的风险代码、慢路径，以及该删什么 |
| `/skill:ponytail-audit` | 同一套检查，范围是整个仓库，按重要性排序 |
| `/skill:ponytail-debt` | 把散落在代码里的 `shortcut:` 注释整理成债务清单 |
| `/skill:ponytail-gain` | 展示原项目基准测得的收益（代码量、成本、速度） |
| `/skill:ponytail-help` | 上述命令的速查表 |

`/ponytail` 只接受合法参数。`review`、`default ultra` 之类的输入会返回用法提示，且不改动配置。

### 级别

| 级别 | 行为 |
| --- | --- |
| `lite` | 直接建造被要求的东西，用一行说明更小的选项，让用户自己选 |
| `full` | 默认。完整规则 |
| `ultra` | 在 `full` 之上，先质疑请求：需求无法支撑的部分动手前先反对 |
| `off` | 不注入任何规则 |

切换级别只改变注入内容，模型下一次请求即生效，不需要重启 rpi。

## 配置

默认级别按以下顺序解析，取第一个合法值：

1. 环境变量 `PONYTAIL_DEFAULT_MODE`（取值为 `off`/`lite`/`full`/`ultra`，忽略大小写与空白）
2. 配置文件 `defaultMode` 字段
3. 内置默认 `full`

配置文件位置：

| 平台 | 路径 |
| --- | --- |
| 任意平台设置了 `XDG_CONFIG_HOME` | `$XDG_CONFIG_HOME/ponytail/config.json` |
| Linux / macOS | `~/.config/ponytail/config.json` |
| Windows（未设 `XDG_CONFIG_HOME`） | `%APPDATA%\ponytail\config.json` |

格式：

```json
{
  "defaultMode": "full"
}
```

该文件与原版 Ponytail 共用，可以同时写其他原版字段（如 `quietStartup`）。

### 持久化行为

`/ponytail <level>` 成功切换后会立即写回配置文件：

- 只更新 `defaultMode`，**文件里的其他字段原样保留**。
- 先写同目录的 `config.json.tmp` 并 `fsync`，再原子 `rename` 覆盖，写入中断不会留下半个文件。
  替换后的文件按进程 umask 重新创建，不会继承原文件的权限位（需要严格控制权限时用 `umask` 或 ACL）。
- 配置文件不是合法 JSON 对象、目录不可写等情况：打印错误，**当前模式保持不变**，配置文件也不被改写。

`PONYTAIL_DEFAULT_MODE` 优先级高于配置文件，所以设了它时切换会提示
`Ponytail mode set to lite. PONYTAIL_DEFAULT_MODE keeps the default at off.`——写入照常发生，但重启后仍以环境变量为准。

## 与原版的差异

原版面向 Claude Code、Codex、pi 等多个宿主，本插件按 rpi 的能力裁剪：

| 原版能力 | 本插件 |
| --- | --- |
| 每轮注入规则 | 支持，注入到现有系统提示词末尾（原样保留宿主已有内容） |
| `/ponytail lite\|full\|ultra\|off` | 支持，且会持久化 |
| `/ponytail status` | 支持 |
| `/ponytail default <mode>` | 不支持。切换即写入默认值，因此没有单独的"只改默认"动作 |
| 英文短语 "stop ponytail" / "normal mode" 停用 | 不支持。用 `/ponytail off` |
| 状态栏指示器、`/ponytail band` | 不支持（Claude Code 专有）、`hideStatus` 配置项无效果 |
| `ponytail-review` 等短命令别名 | 不支持。用 `/skill:ponytail-*` 或让模型按技能描述自行读取 |
| 会话记忆（切换/恢复/分叉后各自恢复模式） | 不支持。模式是插件进程内的一份状态，所有会话共享 |

进程内状态的含义：切换后所有打开的会话立即使用新级别；重启 rpi 时从配置重新读取，
所以重启后的级别是配置文件（或环境变量）里的值，而不是上次会话零散的切换历史。

## 边界与权限

- **文件读写**：只读写上述 Ponytail 配置文件。不触碰项目文件、不写日志。
- **进程与网络**：不启动子进程，不发起网络请求。
- **技能**：技能文本可能指导模型读取或审查项目代码，但实际动作仍受 rpi 自身的工具白名单与权限规则约束；
  插件不绕过它们。
- **能力缺失**：宿主缺少 `register_command` 或 `register_before_agent_start` 槽位时，插件注册直接返回非零，
  由宿主报告加载失败——不会出现"命令可用但规则没注入"的半生效状态。
- **平台**：目标为 Linux、macOS、Windows；目前只在 Linux 上完成真实构建、加载与端到端验证，另两个平台未实测。

## 开发

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

调试时在仓库内运行 `rpi dev`（源码变更自动重编译并热重载）或 `rpi dev-local`（隔离模式，只加载本插件）。

仓库结构：

```text
src/lib.rs       插件全部逻辑：模式解析、配置读写、提示词注入、命令处理
src/ponytail.md  内嵌的原版规则正文（含 front matter），按级别裁剪后注入
skills/          随包分发的原版专项技能
```

`src/ponytail.md` 是原项目 `skills/ponytail/SKILL.md` 的副本。上游更新规则后需要同步替换它并重新构建，
构建时通过 `include_str!` 编入，所以不依赖运行时的文件路径。

规则正文按级别裁剪：`## Levels` 表格中除当前级别外的行会被去掉，其余行（含"Never cut"清单）
原样保留。

## 许可

原版规则与技能依据 MIT 协议复制，原版权归 Dietrich Gebert 所有，完整声明见 [LICENSE](LICENSE)。
