---
# rpi-ponytail-wpr3
title: 发布 rpi-ponytail 0.1.1 预编译包
status: completed
type: task
priority: normal
created_at: 2026-11-09T03:29:04Z
updated_at: 2026-11-09T03:41:02Z
---

用 GitHub Actions 五平台矩阵构建归档，smoke 通过后发布 manifest，最后验证 rpi install 可装。

## Summary of Changes

GitHub Actions 五平台全部通过（release/smoke/manifest），release 已公开。真实 rpi 0.21.1 用 rpi install --github morler/rpi-ponytail --version 0.1.1 装到干净目录后 /ponytail status 返回 'Ponytail: current full • default full'。
