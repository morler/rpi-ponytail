---
# rpi-ponytail-ewwn
title: Fix front matter CRLF leak and temp collision
status: completed
type: bug
priority: normal
created_at: 2026-11-08T01:43:28Z
updated_at: 2026-11-09T03:35:01Z
---

审查发现两处：
- src/lib.rs instructions() 用 strip_prefix("---
")，Windows autocrlf 检出时 front matter 会进入系统提示词
- save_mode 固定临时名 config.json.tmp，跨进程并发切换会互踩

修复：新增 front_matter_body 归一化 CRLF；临时名带 pid；新增 .gitattributes eol=lf；补测试。

## Summary of Changes

front_matter_body 归一化 CRLF 后剥离 front matter；save_mode 临时名带 pid；新增 .gitattributes (* text=auto eol=lf)；新增 strips_front_matter_with_lf_and_crlf 测试与空参数 status 断言。cargo fmt/clippy/test 通过（5 passed）。
