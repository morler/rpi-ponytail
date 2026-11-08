# rpi-ponytail

将 [Ponytail](https://github.com/DietrichGebert/ponytail) 的最小完整改动规则带到 rpi。Rust 原生插件在每轮开始时向现有系统提示词追加对应级别的原版规则，默认 `full`，不访问网络。

## 安装

在目标项目中（将路径替换为本仓库的实际路径）：

```sh
rpi package link --path /path/to/rpi-ponytail
mkdir -p .rpi/skills
cp -R /path/to/rpi-ponytail/skills/ponytail-* .rpi/skills/
```

项目须允许加载本地资源。`rpi package link` 只链接动态库，不会复制这些技能；更新本项目时请重新复制技能。卸载时执行 `rpi package unlink rpi-ponytail`，并删除自行复制的 `.rpi/skills/ponytail-{review,audit,debt,gain,help}`。这不会删除用户的 Ponytail 配置。

## 使用

- `/ponytail lite|full|ultra|off`：切换级别，并写入 Ponytail 配置的 `defaultMode`，重启后仍然生效。
- `/ponytail status`：显示当前和默认级别；无参数时，已启用则显示状态，关闭时启用默认级别（默认关闭则启用 full）。
- `/skill:ponytail-review`、`/skill:ponytail-audit`、`/skill:ponytail-debt`、`/skill:ponytail-gain`、`/skill:ponytail-help`：调用随附的原版专项技能，也可根据描述由模型按需读取。

默认级别依次取合法的 `PONYTAIL_DEFAULT_MODE`、`$XDG_CONFIG_HOME/ponytail/config.json` 的 `defaultMode`、`full`。未设置 XDG 时，Unix 使用 `~/.config`，Windows 使用 `%APPDATA%`。合法值为 `off`、`lite`、`full`、`ultra`。

`/ponytail` 每次成功切换都会把模式写入该配置文件：保留文件内的其他字段，先写 `config.json.tmp` 再原子替换；配置文件不是合法 JSON 对象或写入失败时，当前模式保持不变并报错。`PONYTAIL_DEFAULT_MODE` 优先级更高，设置后即使写入成功，重启仍以环境变量为准（命令会提示这一点）。

## 与原版的差异和边界

当前进程内的模式为所有会话共享，重启后从配置文件恢复；切换、恢复、分叉会话不会恢复各自模式。暂不支持原版的 `/ponytail default`、自然语言停用、状态栏、专项技能的短命令别名或 Claude 专属 band。模式关闭只停止自动注入，不会卸载技能。手动调用技能仍会加载其指令。

插件使用当前用户权限读写 Ponytail 配置；不执行外部命令、不访问网络。技能可能指导模型读取或审查项目文件，实际操作仍由 rpi 的工具及权限规则控制。宿主缺少命令注册或系统提示词变换能力时插件注册失败，不会假装生效。

支持目标为 Linux、macOS、Windows；目前仅在 Linux 上构建和加载验证。开发检查：`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`。

原版技能和规则依据 MIT 协议复制，版权声明见 [LICENSE](LICENSE)。
