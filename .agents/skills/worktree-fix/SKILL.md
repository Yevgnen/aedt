---
name: worktree-fix
description: 仅当用户显式点名本 skill（如「用 worktree-fix」或 $worktree-fix）时调用。在新建的 Git 分支与关联 worktree 中完成用户指定的改动并提交，再合并回任务开始时的原分支：原工作区 dirty 也直接合并、绝不 stash 他人改动；因未提交改动被 git 拒绝时每 30 秒重试、最长 10 分钟；合并成功后删除临时分支与 worktree。
---

# 在分支与 worktree 里干活，再合并回原分支

用户的普通改动、修 bug、重构请求不构成调用授权。只有用户明确点名本 skill 才执行。

整个过程只做用户指定的这一件事。原 worktree 里所有未提交的改动都属于用户，不暂存、不提交、不 stash、不 reset、不 checkout、不 clean、不格式化。

## 1. 记录初始状态

在原 worktree 中：

1. 仓库根绝对路径记为 `original_worktree`。
2. `git symbolic-ref --quiet --short HEAD` 的结果记为 `original_branch` 。HEAD 处于分离状态时停止。
3. 记录 `git status --short` 与当前 HEAD，作为后续判断 dirty 的基线。
4. 读仓库里的说明文件（如 `AGENTS.md`）并遵守。

## 2. 创建分支与 worktree

分支名唯一，例如 `worktree-fix/<任务>-<YYYYMMDD-HHMMSS>` 。用 `mktemp -d "${TMPDIR:-/tmp}/worktree-fix-XXXXXX"` 建目录，然后：

```bash
git -C "$original_worktree" worktree add -b "$task_branch" "$task_worktree" "$original_branch"
```

不复用已有分支，也不往非空目录里加 worktree。全程记住这两个路径和分支名。

## 3. 在 worktree 中完成并提交

编辑、格式化、生成、测试都在 `task_worktree` 里做，不碰 `original_worktree`。

提交前：看完整 diff 与状态，删掉与任务无关的产物，跑与改动匹配的测试。提交信息按 Conventional Commits。合并前必须至少有一个提交。

## 4. 合并回原分支

### 4.1 先确认目标没变

```bash
git -C "$original_worktree" symbolic-ref --quiet --short HEAD
```

不是 `original_branch` 就停止并询问用户，不擅自切换原 worktree 的分支。

### 4.2 直接合并，不 stash

无论原工作区是否 dirty，都直接执行：

```bash
git -C "$original_worktree" merge --no-edit "$task_branch"
```

- 原工作区干净：正常合并，能 fast-forward 就 fast-forward。
- 原工作区 dirty：照常执行上面这条。Git 只会在不会覆盖他人改动时才成功。
- 永远不为了合并去 `git stash` 、`git checkout --` 、`git reset` 、`git clean` ，也不提交别人的改动。

### 4.3 按失败原因分流

捕获命令的 stderr 原文再判断：

| 情况 | 识别特征 | 处理 |
| --- | --- | --- |
| 成功 | exit 0 | 进入第 5 步 |
| 他人的未提交或未跟踪文件会被覆盖 | `would be overwritten by merge` 、`Please commit your changes or stash them` | 进入 4.4 重试 |
| 自动合并产生内容冲突 | `CONFLICT` 、`Automatic merge failed` | 立即停止 |
| 其他错误 | 其余情况 | 报告原文并停止 |

命中内容冲突时，只中止本流程发起的这次合并：

```bash
git -C "$original_worktree" merge --abort
```

若 abort 因为他人改动而失败，就保留现状。无论哪种，都保留 `task_branch` 与 `task_worktree` ，向用户报告冲突路径与原文，等用户处理。

### 4.4 重试等待

只在「文件会被覆盖」这一种失败下重试：

- 间隔 30 秒，总共最多 10 分钟（约 20 次） 。单次等待用一条 `sleep 30` ，terminal 工具设 `timeout_ms` 覆盖这段等待。
- 每次重试前重新确认原 worktree 仍在 `original_branch` ，并重新看 `git status --short` ，因为等待期间原工作区可能已经被用户清理干净或切换到别的分支。
- 等待期间不改动原 worktree ：不 stash、不提交、不切换分支、不删除未跟踪文件。
- 每轮向用户报一句：第几次、当前 dirty 路径、上一次 Git 的报错。
- 10 分钟仍失败就停止，贴出被拒路径和 Git 原文，把 `task_branch` 与 `task_worktree` 留给用户。

### 4.5 确认合并结果

```bash
git -C "$original_worktree" merge-base --is-ancestor "$task_branch" "$original_branch"
```

返回 0 才算合并成功。

## 5. 清理

只有合并成功且上面的祖先检查通过，才执行：

```bash
git -C "$original_worktree" worktree remove "$task_worktree"
git -C "$original_worktree" branch -d "$task_branch"
```

不用 `--force` ，不用 `-D` 。worktree 里还有未提交或未跟踪（非忽略）的文件时，说明原因并停下，不删文件。清理失败就保留并报告具体路径或分支名。

## 6. 汇报

分三段：实际改动（文件与前后行为）、验证结果（命令与输出摘要）、未完成或未验证项。合并与清理各自的结果都要写清楚：合并是 fast-forward 还是产生了合并提交，删掉了哪个分支和哪个 worktree。
