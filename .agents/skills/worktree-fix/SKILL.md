---
name: worktree-fix
description: 仅当用户显式点名本 skill（如「用 worktree-fix」或 $worktree-fix）时首次启用；启用后同一会话里的后续任务除非用户指明不用，都默认按本流程开始。在新建的 Git 分支与关联 worktree 中完成用户指定的改动并提交，再合并回任务开始时的原分支：原工作区 dirty 也直接合并、绝不 stash 他人改动；因未提交改动被 git 拒绝时每 30 秒重试、最长 10 分钟；内容冲突能在任务 worktree 里明确解决时先把原分支合进来再重试，解决不明了才停下；合并成功后删除临时分支与 worktree。
---

# 在分支与 worktree 里干活，再合并回原分支

## 触发与默认沿用

- 首次启用只能由用户显式点名本 skill（例如「用 worktree-fix」） ，普通改动、修 bug、重构请求都不构成授权。
- 一旦启用，同一会话里后续所有任务都默认按本流程开始，不必再次点名。
- 用户明确说「直接改」「不用 worktree」「就在当前分支做」之类的，只覆盖他指明的那一个任务，之后的任务仍回到默认。
- 每个任务都要重新走第 1 到第 6 步：重新记录 `original_worktree` 与 `original_branch`（可能已经变了） ，新开分支和 worktree，不复用上一次的。
- 新开一个会话不会自动加载本文件，那里的第一次启用仍需用户点名；要让默认跨会话生效，得把这条规则写进项目的 `AGENTS.md` ，本 skill 管不到。

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
| 他人的未提交或未跟踪文件会被覆盖 | `would be overwritten by merge` 、`Please commit your changes or stash them` | 进入 4.5 重试 |
| 自动合并产生内容冲突 | `CONFLICT` 、`Automatic merge failed` | 进入 4.4 |
| 其他错误 | 其余情况 | 报告原文并停止 |

命中内容冲突时不要在 `original_worktree` 里就地解决，按 4.4 处理。

### 4.4 冲突能明确解决时，先把原分支合进任务分支

冲突一律在 `task_worktree` 里解决，`original_worktree` 里的他人改动一概不碰。

1. 只中止本流程发起的这次合并：

   ```bash
   git -C "$original_worktree" merge --abort
   ```

   abort 因为他人改动而失败时，保留现状并报告，不要改用更激进的手段清场。

2. 在任务 worktree 里把原分支合进来：

   ```bash
   git -C "$task_worktree" merge --no-ff "$original_branch"
   ```

3. 逐个看冲突文件，只有不需要业务判断的才算「明确可解」：
   - 两边只是同一文件里不同位置的新增，互不相干；
   - 同一函数内的相邻改动，两边意图可以直接叠加，例如一边改 import 、另一边加调用；
   - 一边只是格式、排序或改名，另一边是实质改动，按实质改动保留；
   - 生成类文件（如 `Cargo.lock` ）两边都动过，可以按仓库既有方式重新生成，并在任务 worktree 的测试里确认结果可用。

   下面这些视为不明了：
   - 同一行或同一代码分支上两个互斥的取值、默认值、条件判断；
   - 同一段代码一边删除一边修改；
   - 涉及公开 API 、行为语义、数据格式或迁移的取舍；
   - 要靠猜业务规则才能决定保留哪边。

4. 明确可解时：直接编辑冲突文件保留双方意图，`git add` 后用 `git -C "$task_worktree" commit --no-edit` 完成这次合并，不要在任务分支上提交猜出来的结果。
5. 在任务 worktree 里跑与改动匹配的构建或测试，确认解决后仍能编译、行为符合预期。
6. 回到 4.2 重新合并回原分支。原分支可能又前进了，重复 4.2 到 4.4 即可，最多重复 3 轮；超出就停下报告，不无限追着原分支合。

冲突不明了，或解决后构建、测试失败且无法确定原因时：立即停止，保留 `task_branch` 与 `task_worktree` ，贴出冲突路径、两边原始内容与 Git 原文，等用户决定。

### 4.5 重试等待

只在「文件会被覆盖」这一种失败下重试：

- 间隔 30 秒，总共最多 10 分钟（约 20 次） 。单次等待用一条 `sleep 30` ，terminal 工具设 `timeout_ms` 覆盖这段等待。
- 每次重试前重新确认原 worktree 仍在 `original_branch` ，并重新看 `git status --short` ，因为等待期间原工作区可能已经被用户清理干净或切换到别的分支。
- 等待期间不改动原 worktree ：不 stash、不提交、不切换分支、不删除未跟踪文件。
- 每轮向用户报一句：第几次、当前 dirty 路径、上一次 Git 的报错。
- 10 分钟仍失败就停止，贴出被拒路径和 Git 原文，把 `task_branch` 与 `task_worktree` 留给用户。

### 4.6 确认合并结果

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

分三段：实际改动（文件与前后行为）、验证结果（命令与输出摘要）、未完成或未验证项。合并与清理各自的结果都要写清楚：合并是 fast-forward 还是产生了合并提交，是否因冲突把原分支合进过任务分支、冲突是怎么解的，删掉了哪个分支和哪个 worktree。

不管流程是跑完还是中途停下，回复的最后一行都单独输出一行状态，每项前按实际结果加 tick 或 cross emoji：

✅ 已合并 ✅ 已删除分支 ✅ 已删除 worktree

这一行是回复的末尾，后面不再写任何内容。某项没有执行或失败就写成 ❌ ，并在上面的「未完成或未验证项」里说明原因。
