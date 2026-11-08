---
# rpi-ponytail-knut
title: 持久化 Ponytail 模式
status: completed
type: task
priority: normal
created_at: 2026-11-08T01:20:18Z
updated_at: 2026-11-08T01:23:55Z
---

设置模式时安全写入配置并验证错误处理。

## Summary of Changes

/ponytail 切换模式时原子写入 Ponytail 配置 defaultMode，保留其他字段；写入失败不改当前模式。单元测试 4 项通过，并在临时项目中经 RPC 端到端验证持久化、重启恢复与错误路径。
