# Blend Contract V2 Instructions

## Overview

Blend is a liquidity protocol primitive on Stellar. 

For a detailed description, view the [Whitepaper](#whitepaper) section below.

## Build and Test Commands
* `make build` - Compiles the Blend contracts.
* `make test` - Runs the test suite for the Blend contracts.

## Project Structure
* `backstop/` - Contains the backstop module contract
* `pool/` - Contains the isolated lending pool contract
* `pool-factory/` - Contains the factory contract for deploying isolated lending pools
* `emitters/` - Contains the compiled emitter contract for emitting BLND tokens
* `mocks/` - Contains contracts used in testing
* `test-suites/` - Contains the integration test suites for the Blend protocol
* Unit tests live in most source files where applicable.

## Coding Style
* Write idiomatic Rust code
* Keep the style consistent with the existing codebase
* Add comments and documentation for complex logic

## Whitepaper

### Abstract

This paper introduces a liquidity protocol primitive, Blend. A liquidity protocol primitive enables the permissionless creation of lending pools to quickly respond to emerging market needs. Applications, industries, and users can utilize this primitive to create isolated lending pools that best serve their niche. Blend is an ungoverned, modular, liquidity protocol primitive that does not compromise on security or capital efficiency.

### Introduction

Decentralized money markets act as a cornerstone for healthy crypto-economic systems. They trustlessly facilitate the flow of capital to wherever it is most productive, increasing capital efficiency and generating interest along the way. Aave and Compound prove these products' value with their success in the Ethereum ecosystem. Since their inception during the early stages of decentralized finance (DeFi), they quickly became two of the industry’s largest and most used DeFi protocols. Aave remains one of the largest, peaking at approximately $30 billion in liquidity \[[1](https://github.com/aave/aave-v3-core/blob/master/techpaper/Aave_V3_Technical_Paper.pdf)].

Despite their usefulness, current money markets fall short in terms of flexibility. Users want to utilize a wide range of their crypto assets in money markets. However, supporting risky assets, especially as collateral, can put protocol funds at risk. Aave and Compound forgo flexibility and have extensive governance systems that ensure any asset meets well-defined criteria before adding it to their markets \[[2](https://medium.com/gauntlet-networks/gauntlets-parameter-recommendation-methodology-8591478a0c1c), [3](https://docs.aave.com/risk/asset-risk/introduction)]. Other protocols like Euler and Rari have novel approaches for managing permissionless listings that segment asset risk \[[4](https://docs.euler.finance/getting-started/white-paper#permissionless-listing)]. Unfortunately, these approaches can lead to liquidity fragmentation and low capital utilization.

Blend represents a new, more primitive approach to decentralized money market protocols. It enables the permissionless creation of isolated lending pools, ensuring maximum flexibility and accessibility. Blend avoids the normal pitfalls of permissionless lending markets by using a market-driven backstop system to curate pools, ensuring appropriate risk levels, and using a dynamic interest rate model to preserve capital efficiency.

### Protocol Specification

#### Isolated Lending Pools

Blend’s core component is its isolated lending pools. These pools facilitate lending and borrowing between users for a set of supported assets. Anyone can deploy an isolated pool using Blend, allowing the protocol to quickly adapt to the needs of constantly evolving digital economies. The deployer sets parameters that govern the pool’s supported assets, acceptable loan-to-value ratios, target utilization rates, and price oracle.

Blend ensures asset risk segmentation by logically separating positions in isolated pools from all other pools, meaning collateral and debt associated with one isolated pool does not apply to others. Thus, bad debt, liquidations, or bad oracles in one pool cannot spill over and harm users in another pool. As a result, the adaptability offered by Blend’s permissionless deployment does not expose protocol users to unknown risks.

There are two types of isolated pools, standard and owned:

#### Backstop Module

The backstop module is a pool of funds that acts as first-loss capital for each isolated lending pool. Each pool's backstop funds are specific to themselves, and used extensively in pool management.

##### Depositing and Withdrawing Funds

Users deposit 80/20 weighted BLND:USDC liquidity pool tokens into a backstop module for an isolated lending pool. BLND being 80% of the pool's weight, and USDC being 20. They can withdraw their deposits at any time. However, initiating a withdrawal places the funds into a withdrawal queue, where they remain for 17 days. Deposits that are queued for withdrawal will not earn backstop emissions. After the queue expires, users can withdraw their funds as long as the backstop module has no remaining bad debt. The queue period ensures the backstop module can effectively perform its function as lender insurance.

#### Lending and Borrowing

##### Lending Assets

Any asset supported by a given pool can be deposited into that pool. A deposit is represented by an ERC-20 token, or bToken, which entitles the holder to a share of the total deposited assets (similar to Compound’s cTokens \[[5](https://compound.finance/docs/ctokens)]).

The lender receives bTokens for depositing amount of an asset based on the following exchange rate:

$$
bTokens = \frac{amount}{bTokenRate}
$$

The bTokenRate is an internal value that tracks how much interest has accrued to one bToken over the pool’s lifetime. For example, if 10% interest has been generated by a bToken, its bTokenRate will be 1.1.

##### Borrowing Assets

Any asset supported by the pool can be borrowed from the pool as long as the borrower has sufficient collateral deposited. In the event of a price change, borrowers risk liquidation if the collateral they have posted is no longer sufficient to cover their outstanding liabilities.

Borrowed assets are tracked with an ERC-20 token, or dToken (similar to Aave’s debtToken), which represents an outstanding liability for the holder against the pool for the borrowed token. These tokens are non-transferable and can only be removed by repaying the borrowed amount to the pool.

The borrower receives dTokens for borrowing amount of an asset from the pool based on the following exchange rate:

$$
dTokenRate = \frac{(bTokenTotalSupply \* bTokenRate - PoolAssetBalance)}{dTokenTotalSupply}
$$

$$
dTokens=\frac{amount}{dTokenRate}
$$

The value of assets a user can borrow from a pool is based on their Borrow Limit. Each collateral position in the pool increases their borrow limit by the value of the position multiplied by its Collateral Factor. Each liability position decreases their borrow limit by the value of the position divided by its Liability Factor. Each asset's collateral and liability factor is set on pool creation and bounded within \[0.00, 1.00].

$$
Borrow Limit = \sum(PositionValue\_{collateral} \* AssetFactor\_{collateral}) -\sum(\frac{PositionValue\_{Liability}}{AssetFactor\_{Liability}})
$$

Supporting both a collateral and liability factor gives pool creators a large amount of flexibility regarding the level of leverage they permit for different positions. For example, if a pool creator wants to support high leverage for fiat borrows collateralized with fiat, but not crypto borrows collateralized with fiat, they can set all fiat collateral and liability factors to 0.98 and crypto collateral and liability factors to 0.765. This would give fiat borrowers using fiat as collateral up to 25x leverage, but crypto borrowers using fiat as collateral only up to 4x leverage. This level of flexibility is not possible using only collateral factors.

#### Liquidations

In the event a borrower’s outstanding liabilities to a given pool exceed the borrowing capacity of their collateral in that pool, the borrower can be liquidated. This transfers a portion of the delinquent user's collateral and liability positions to a liquidator's account, where the liquidator can handle them as they wish. Liquidations are performed to reduce the chance a borrower defaults on their loan, causing a permanent loss of lender funds.

Blend uses gentle dutch auctions to liquidate borrowers without over-penalizing them or exposing them to oracle risk.

##### Auction Initiation

Any user can initiate a liquidation auction on an account that has exceeded its borrow capacity for a pool. Liquidation initiators choose a percent,$L\_p$, of user liabilities to be auctioned off (the bid of the auction). Based on that percentage, the protocol will set a percentage of the user's collateral to also be auctioned (the lot of the auction). The collateral percentage, $C\_p$ is calculated using an estimated liquidator premium, $p$, the excess collateral value a liquidator is expected to demand in order to fill the auction:

$$
p = \frac{1-\overline{C\_f}\*\overline{L\_f}}{2}+1
$$

$$
C\_p = \frac{p*L\_p*L\_o}{C\_o}
$$

$$
\begin{align\*} \text{where}\ \&p=\text{Estimated Liquidator Premium}\ &\overline{L\_f}=\text{Average liability factor of the liability positions}\ &\overline{C\_f}=\text{Average collateral factor of the collateral positions}\\\&Lo=\text{Total value of the user's liabilities outstanding}\ \&C\_o=\text{Total value of the user's collateral outstanding}\\\&Lp=\text{Percent of liability positions being auctioned off}\ \&C\_p=\text{Percent of collateral positions being auctioned off}\ \end{align\*}
$$

When creating the liquidation, the protocol requires that the creator inputs an $Lp$ that ensures that if the liquidation is 100% cleared (all liability and collateral positions are transferred to the liquidator), after liquidation, the liquidated user's health factor is between 1.03 and 1.15.

##### Auction Duration

After the auction initiation, a lot modifier ($LM$) value begins scaling from zero to one based on how much time has passed since initiation. This value governs the percent of the auctioned user’s collateral the liquidator will receive upon filling the auction (1.00 being 100%). Once the lot modifier reaches one, a bid modifier ($BM$) value begins decreasing from one to zero. The bid modifier governs what percentage of the input liability amount the liquidator must repay when they fill the auction (1.00 being 100%). These modifiers are calculated based on the number of blocks that have passed since the auction was created:

$$
LM(\Delta b)= \begin{cases} \lfloor \frac{\Delta b}{2} \rfloor & \text{if } \Delta b\leq 200\ 1 & \text{if } \Delta b\gt 200\ \end{cases}, \hspace{2mm} BM(\Delta b)= \begin{cases} 1 & \text{if } \Delta b\leq 200\ \lfloor \frac{\Delta b}{2} \rfloor& \text{if } \Delta b\gt 200\ \end{cases}
$$

$$
\begin{align\*} \text{where}\ &\Delta b=\text{The number of blocks that have passed since the auction was initialized}\ \end{align\*}
$$

##### Auction Fill

At any point, a liquidator can fill the auction and assume the bid modifier adjusted auction liability as well as the lot modifier adjusted auctioned collateral. After filling the liquidation, the liquidator must ensure their account is in a healthy state with a health factor greater than 1. This means they may need to repay the assumed liabilities or post more collateral.

