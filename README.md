# ATX Points Exchange

这是根据需求文档生成的一套 Anchor 0.29.0 合约骨架，包含：

- `programs/atx_points_exchange/src/lib.rs`：Solana Rust 智能合约
- `client/atx-points-exchange.ts`：前端调用工具
- `client/lovable-withdraw-button.tsx`：可直接嵌入 Lovable/React 页面“提现”按钮的示例组件

## 已实现的链上能力

- 初始化全局配置 `initialize_config`
- 管理员增加积分 `add_user_points`
- 管理员扣减积分 `sub_user_points`
- 管理员暂停或恢复兑换 `toggle_pause`
- 用户兑换积分 `exchange_points`
- 管理员提取 ATX `withdraw_tokens`

## 重要说明

1. Solana 合约本身不能像传统后端 API 一样直接返回动态数组，所以文档中的：
   - `get_user_points(user_pubkey)`
   - `get_exchange_records(user_pubkey)`
   - `get_global_config()`

   这里已经通过 `client/atx-points-exchange.ts` 里的账户查询方法来实现，前端直接读 PDA 账户即可。

2. 当前程序 ID 使用的是 Anchor 默认占位值：

   - `programs/atx_points_exchange/src/lib.rs`
   - `Anchor.toml`

   在你正式部署前，必须替换成你自己的 Program ID。

3. `initialize_config` 会自动创建合约金库的 ATX Token Account。
   管理员需要先把足量的 ATX 充值到该金库账户，用户兑换时才能成功转账。

## 编译与部署

### 1. 安装依赖

- Rust 1.75+
- Solana CLI
- Anchor CLI 0.29.0

### 2. 编译

```bash
anchor build
```

### 3. 部署

```bash
anchor deploy
```

部署完成后，把新的 Program ID 同步替换到：

- `programs/atx_points_exchange/src/lib.rs`
- `Anchor.toml`
- 你的前端环境变量或 Lovable 配置

### 4. 初始化配置

部署后，管理员钱包需要调用 `initialize_config`，并传入：

- `atx_mint`

### 5. 前端按钮接入

在页面中引入：

```tsx
import { AtxWithdrawButton } from "./client/lovable-withdraw-button";
```

然后向组件传入：

- `connection`
- `wallet`
- `programId`
- `atxMint`

按钮点击后会：

1. 查询当前钱包积分
2. 判断是否大于等于 1000
3. 拉起钱包签名
4. 调用链上 `exchange_points`

## 后续建议

- 增加 Anchor 测试文件，覆盖积分不足、暂停状态、金库余额不足等场景
- 把管理员接口再包一层后台服务，避免直接在前端暴露管理操作
- 部署前补齐生产环境 Program ID、Mint 地址和 RPC 配置
