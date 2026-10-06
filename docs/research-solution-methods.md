# Research: Solution Methods for Simultaneous-Move Tug-of-War

## Overview

This document surveys solution methods from the literature that fit the tug-of-war model as it grows from the basic level to the full realistic game (see `model-evolution.md`). Each method gets:

- **Idea**: what it does, in one paragraph
- **Fits**: which model level(s) it suits, and why
- **Cost / limits**: complexity and known weaknesses
- **Source**: full citation with a link that was checked

The motivation is a modelling gap in the current solvers. `src/algorithms/dp.rs`, `fatigue_dp.rs` and `approximate_dp.rs` all use the sequential pure-strategy recurrence

```text
V_t(S) = max_{a_A ∈ A_A} min_{a_B ∈ A_B} V_{t+1}(T(S, a_A, a_B))
```

In the real game both teams choose their allocation **at the same time**. That makes each turn a matrix game (a Colonel Blotto-like allocation game), and the optimal play may be **mixed**. The sequential maximin above solves the game in which Team A commits first and Team B sees that choice before replying. That value is a lower bound for A, and it is often strictly pessimistic (see the final section).

All citations below were checked against a publisher, proceedings or arXiv page. None are marked unverified.

---

## 1. Colonel Blotto (the per-turn game)

### Idea
Two players with fixed force budgets split them across `n` battlefields at the same time. Whoever puts more force on a battlefield wins it, and the payoff is the number (or share) of battlefields won. Borel stated the problem in 1921. Roberson (2006) characterised the equilibrium payoffs for both symmetric and asymmetric budgets. In general the equilibria are mixed: a player's allocation must be unpredictable, otherwise the opponent beats it with a slightly larger force on enough ropes.

### Fits
- **Level 3 (two teams)** and above: one turn of tug-of-war is a discrete Blotto game. Players are the units, ropes are the battlefields, and rope movement replaces win/lose. The terminal score makes it a finite-horizon *repeated / dynamic* Blotto game.
- Useful as a **theory check**: in one-shot tests the solver should reproduce known Blotto behaviour, such as no pure saddle point when budgets are close.

### Cost / limits
- The discrete Blotto game has `C(P+R-1, R-1)` pure strategies per side (stars and bars, see `complexity.md`). Solving one turn exactly is an LP of that size, which is fine for small `P, R`.
- Roberson's closed forms assume continuous forces and the "win share of battlefields" payoff. Integer players, rope positions and fatigue break the closed forms, so the theory guides the design but does not hand over a solver.

### Sources
- E. Borel, "La théorie du jeu et les équations intégrales à noyau symétrique gauche", *Comptes Rendus de l'Académie des Sciences* 173, 1304–1308, 1921. English translation by L. J. Savage: "The Theory of Play and Integral Equations with Skew Symmetric Kernels", *Econometrica* 21(1), 97–100, 1953. https://www.econometricsociety.org/publications/econometrica/1953/01/01/theory-play-and-integral-equations-skew-symmetric-kernels (JSTOR: https://www.jstor.org/stable/1906947)
- B. Roberson, "The Colonel Blotto game", *Economic Theory* 29(1), 1–24, 2006. https://doi.org/10.1007/s00199-005-0071-5 (https://link.springer.com/article/10.1007/s00199-005-0071-5)

---

## 2. Shapley Stochastic Games (exact DP done right)

### Idea
Shapley defined two-player zero-sum games that move between states. At each state both players choose actions simultaneously, receive a payoff, and the game transitions, possibly at random. He proved the game has a value and that it is the fixed point of a Bellman-like operator. That operator replaces `max min` over pure actions with the **value of the matrix game** at each state:

```text
V_t(S) = val[ M_S ],   M_S[a_A][a_B] = r(S, a_A, a_B) + E[ V_{t+1}(S') ]
val[M] = max_{x ∈ Δ(A_A)} min_{b ∈ A_B} Σ_a x_a M[a][b]      (solved by LP)
```

Our horizon is finite, so this is plain backward induction with one LP per state. No value-iteration convergence argument is needed.

### Fits
- **Levels 3–7** (two teams, fatigue, rope weight, heterogeneous players, switching costs): the state is fully observed and the moves are simultaneous. This is the **direct, correct replacement** for the current exact solvers. It reuses the existing state enumeration and swaps the inner `max/min` loop for an LP solve (or a fast matrix-game solver such as fictitious play or regret matching for approximate values).
- **Level 8 (commander delay)** still fits once the pending-command queue is part of the state.

### Cost / limits
- Time is roughly `|S| × LP(|A_A|, |A_B|)`, against `|S| × |A_A| × |A_B|` today. For 3v3 on 2 ropes (4×4 matrices) the extra cost is negligible. For 12v12 on 3 ropes, `|A| = C(14,2) = 91`, which gives 91×91 LPs, still feasible per state, but the state space with energy dimensions is not.
- It needs full observability, so it does not apply directly to Levels 9–10.

### Source
- L. S. Shapley, "Stochastic Games", *Proceedings of the National Academy of Sciences* 39(10), 1095–1100, 1953. https://doi.org/10.1073/pnas.39.10.1095 (PubMed Central copy: https://pmc.ncbi.nlm.nih.gov/articles/PMC1063912/; reprint listing: https://www.degruyterbrill.com/document/doi/10.1515/9781400829156-013/html)

---

## 3. Simultaneous-Move MCTS (SM-MCTS)

### Idea
Monte Carlo tree search over joint-action nodes. Each node keeps separate statistics for each player's own actions and selects them with a bandit rule. The decoupled UCT variant runs independent UCB per player. Alternatives such as Exp3 and regret matching are Hannan-consistent. Lisý et al. (2013) proved that with an ε-Hannan-consistent selection rule and enough exploration, SM-MCTS converges to an approximate Nash equilibrium. Plain decoupled UCT carries no such guarantee and can fail to converge. Bošanský et al. (2016) compared exact backward induction (with pruning) against sampling methods for this whole class of games.

### Fits
- **Levels 4–8** at sizes where exact Shapley DP runs out of memory, for example 6v6 or 12v12 with fatigue. It is an anytime online planner: it searches from the current state only.
- It is a natural next step after `approximate_dp.rs`. It keeps simultaneous moves, which the current approximate DP collapses into sequential `max min`.

### Cost / limits
- Cost is per decision (iterations × rollout length), with no global table. Quality depends on the iteration budget and the rollout policy.
- Use Exp3 or regret matching at nodes, not plain UCB. The final strategy is the **average** visit distribution, not the most-visited action.
- It assumes perfect information about the state. It does not handle Levels 9–10 directly.

### Sources
- M. Lanctot, V. Lisý, M. H. M. Winands, "Monte Carlo Tree Search in Simultaneous Move Games with Applications to Goofspiel", *Computer Games Workshop (CGW 2013)*, CCIS 408, 28–43, Springer, 2014. https://link.springer.com/chapter/10.1007/978-3-319-05428-5_3
- V. Lisý, V. Kovařík, M. Lanctot, B. Bošanský, "Convergence of Monte Carlo Tree Search in Simultaneous Move Games", *NeurIPS (NIPS) 26*, 2013. https://papers.nips.cc/paper_files/paper/2013/hash/1579779b98ce9edb98dd85606f2c119d-Abstract.html (arXiv: https://arxiv.org/abs/1310.8613)
- B. Bošanský, V. Lisý, M. Lanctot, J. Čermák, M. H. M. Winands, "Algorithms for computing strategies in two-player simultaneous move games", *Artificial Intelligence* 237, 1–40, 2016. https://doi.org/10.1016/j.artint.2016.03.005 (https://cris.maastrichtuniversity.nl/en/publications/algorithms-for-computing-strategies-in-two-player-simultaneous-mo/)

---

## 4. Counterfactual Regret Minimization (CFR) and Successors

### Idea
**CFR** splits overall regret in an extensive-form game into per-information-set "counterfactual regret". It runs regret matching at each information set in self-play, and the **average** strategy converges to a Nash equilibrium in two-player zero-sum games. A simultaneous move fits naturally: B's decision node sits in the same information set as A's, because B has not seen A's choice.

**Deep CFR** replaces the tables with neural networks trained on sampled regrets, so the game needs no hand-built abstraction.

**ReBeL** carries AlphaZero-style self-play + search over to imperfect-information games. It works on *public belief states*, meaning the public state plus each player's probability distribution over private information. It runs CFR-style search in subgames with a learned value network, and it provably converges to Nash in two-player zero-sum games.

### Fits
- **Levels 9–10 (partial information, spies)**: these are the levels where tabular DP stops making sense. Hidden spy identities and local observations create information sets, which is exactly what CFR handles. Tabular CFR (or MCCFR) suits small configurations such as 3v3 with 2 ropes and a short horizon.
- **Deep CFR** for medium sizes where information sets cannot be enumerated.
- **ReBeL** for the full realistic game (Level 11) if hidden information is kept. It is the "end-state" research target, not a first implementation.

### Cost / limits
- Tabular CFR costs `O(|game tree|)` per iteration, and the average strategy converges at roughly `O(1/√T)`. The tree is exponential in the horizon, because there is no state merging across histories unless the game has perfect recall over a compact public state.
- Deep CFR / ReBeL need neural-network training infrastructure (outside the current Rust scope), and ReBeL's guarantees assume two-player zero-sum, which holds here.

### Sources
- M. Zinkevich, M. Johanson, M. Bowling, C. Piccione, "Regret Minimization in Games with Incomplete Information", *NeurIPS (NIPS) 20*, 2007. https://papers.nips.cc/paper/3306-regret-minimization-in-games-with-incomplete-information
- N. Brown, A. Lerer, S. Gross, T. Sandholm, "Deep Counterfactual Regret Minimization", *Proc. ICML 2019*, PMLR 97, 793–802, 2019. https://proceedings.mlr.press/v97/brown19b.html
- N. Brown, A. Bakhtin, A. Lerer, Q. Gong, "Combining Deep Reinforcement Learning and Search for Imperfect-Information Games", *NeurIPS 33*, 2020. https://proceedings.neurips.cc/paper/2020/hash/c61f571dbd2fb949d3fe5ae1608dd48b-Abstract.html (arXiv: https://arxiv.org/abs/2007.13544)

---

## 5. Policy-Space Response Oracles (PSRO)

### Idea
PSRO keeps a population of policies for each player and builds an empirical "meta-game" payoff matrix by simulating every pair. It solves that matrix for a meta-strategy (for example a Nash mixture) and then trains a new **best response** (an oracle, often RL) against the opponent's meta-strategy. The new policy joins the population and the loop repeats. It generalises the double oracle method and fictitious play. The paper also shows that independent RL overfits to its training partners (joint-policy correlation).

### Fits
- **Levels 6–11**, when the action space and state space are too big for tables but the strategies can be written as *parameterised heuristics*. The project already has hand-written strategies in `src/strategies/` that can seed the population.
- It answers the practical question "which mix of our existing strategies is hardest to beat?", using the tournament machinery in `src/experiments/`.

### Cost / limits
- Each iteration needs `O(|population|)` new simulations to fill the matrix, plus one best-response training run. The best-response oracle is the expensive part.
- An approximate best response gives only an approximate equilibrium, and the population can grow large before the meta-Nash stabilises.

### Source
- M. Lanctot, V. Zambaldi, A. Gruslys, A. Lazaridou, K. Tuyls, J. Pérolat, D. Silver, T. Graepel, "A Unified Game-Theoretic Approach to Multiagent Reinforcement Learning", *NeurIPS (NIPS) 30*, 2017. https://proceedings.neurips.cc/paper/2017/hash/3323fe11e9595c09af38fe67567a9394-Abstract.html (arXiv: https://arxiv.org/abs/1711.00832)

---

## 6. AlphaZero and MuZero

### Idea
**AlphaZero** learns a policy and value network purely from self-play. It uses MCTS guided by the network both to choose moves and to produce improved policy targets. **MuZero** removes the need for a known simulator: it learns a latent dynamics model that predicts reward, policy and value, and plans with MCTS inside that model.

### Fits
- **Level 11 at scale** (12v12, 3 ropes, 60 turns, fatigue), where even SM-MCTS without a learned prior is too shallow.
- Caveat: both were designed for **alternating-move, perfect-information** games, and their MCTS selects one action per node. Applied naively to tug-of-war they reproduce the same sequential-commit bias as the current DP. They must be combined with simultaneous-move node handling (Section 3: per-player bandits, or a matrix-game solve at each node) before they are sound here.
- We have an exact simulator, so MuZero's learned model adds little. AlphaZero-style (known model) is the better fit.

### Cost / limits
- Large compute for self-play training, and network training is outside the Rust codebase.
- No equilibrium guarantee in simultaneous or imperfect-information settings without the modifications above.

### Sources
- D. Silver, T. Hubert, J. Schrittwieser, I. Antonoglou, M. Lai, A. Guez, M. Lanctot, L. Sifre, D. Kumaran, T. Graepel, T. Lillicrap, K. Simonyan, D. Hassabis, "A general reinforcement learning algorithm that masters chess, shogi, and Go through self-play", *Science* 362(6419), 1140–1144, 2018. https://www.science.org/doi/10.1126/science.aar6404
- J. Schrittwieser, I. Antonoglou, T. Hubert, K. Simonyan, L. Sifre, S. Schmitt, A. Guez, E. Lockhart, D. Hassabis, T. Graepel, T. Lillicrap, D. Silver, "Mastering Atari, Go, chess and shogi by planning with a learned model", *Nature* 588, 604–609, 2020. https://www.nature.com/articles/s41586-020-03051-4 (arXiv: https://arxiv.org/abs/1911.08265)

---

## 7. POMCP (Partially Observable Monte-Carlo Planning)

### Idea
POMCP runs online MCTS over **histories** for a single agent in a POMDP. It represents the belief state with an unweighted particle filter, updated by Monte-Carlo simulation, and needs only a black-box generative simulator. Explicit transition and observation probabilities are not required.

### Fits
- **Levels 9–10, single-team view**: when the opponent's policy is *fixed or modelled*, for example one of the existing `src/strategies/`, our team faces a POMDP. Spy identities and unseen ropes are the hidden state, and POMCP's particles become the belief `B_t` over "who is a spy" described in `model-evolution.md`.
- It is a cheap, practical step before a full game-theoretic method such as CFR or ReBeL.

### Cost / limits
- Cost is per decision (simulations × depth), and it needs enough particles to cover plausible spy assignments. Particle deprivation is a known risk.
- It is **not** game-theoretic: it best-responds to an assumed opponent model and can be exploited by an adaptive opponent. Treat it as a strong baseline, not an equilibrium solver.

### Source
- D. Silver, J. Veness, "Monte-Carlo Planning in Large POMDPs", *NeurIPS (NIPS) 23*, 2010. https://papers.nips.cc/paper/4031-monte-carlo-planning-in-large-pomdps (author copy: https://dspace.mit.edu/bitstream/handle/1721.1/100395/Silver_Monte-carlo.pdf)

---

## 8. Cooperative MARL Inside a Team: QMIX and MAPPO

### Idea
These methods treat **each player as an agent** and learn decentralised policies with centralised training.
- **QMIX** factorises the team Q-value into per-agent utilities through a monotonic mixing network, so each agent's greedy action is consistent with the joint greedy action.
- **MAPPO** is PPO with a centralised value function. It is a strong, simple baseline on the StarCraft (SMAC), Hanabi, MPE and Google Research Football benchmarks.

### Fits
- **Levels 6–10 with decentralised execution**: heterogeneous players with their own stamina, switching costs per player, commander delay, and local observations (Level 9 says players observe only their own rope). In those settings "team allocation" is really many players deciding locally, and that is what these methods model.
- Combine with an adversarial outer loop: train against a fixed opponent pool, or use MAPPO/QMIX as the **best-response oracle inside PSRO** (Section 5). On its own, cooperative MARL against a fixed opponent is a best response, not an equilibrium.

### Cost / limits
- Training cost is high and needs a Python/NN stack or bindings.
- QMIX's monotonicity cannot represent coordination patterns where one agent's best action depends non-monotonically on teammates. In tug-of-war, "only pull rope 2 if exactly two teammates also do" is one such pattern.
- MAPPO's results are empirical: there is no convergence guarantee in the general case.

### Sources
- T. Rashid, M. Samvelyan, C. Schroeder de Witt, G. Farquhar, J. Foerster, S. Whiteson, "QMIX: Monotonic Value Function Factorisation for Deep Multi-Agent Reinforcement Learning", *Proc. ICML 2018*, PMLR 80, 4295–4304, 2018. https://proceedings.mlr.press/v80/rashid18a.html (arXiv: https://arxiv.org/abs/1803.11485)
- C. Yu, A. Velu, E. Vinitsky, J. Gao, Y. Wang, A. Bayen, Y. Wu, "The Surprising Effectiveness of PPO in Cooperative Multi-Agent Games", *NeurIPS 35 (Datasets and Benchmarks Track)*, 2022. https://papers.nips.cc/paper_files/paper/2022/hash/9c1535a02f0ce079433344e14d910597-Abstract-Datasets_and_Benchmarks.html (arXiv: https://arxiv.org/abs/2103.01955)

---

## Progression Table: Model Level -> Recommended Method

| Level (see `model-evolution.md`) | Game type | Recommended method | Fallback / scale-up |
|---|---|---|---|
| 0–2 Basic, multi-rope, multi-player | Single agent, deterministic | Standard finite-horizon DP (current) | — |
| 3 Two teams | Simultaneous zero-sum, full info | **Shapley backward induction, one LP per state** | Regret matching per state |
| 4–5 Fatigue, rope weight | Same + energy state | Shapley backward induction | SM-MCTS (Exp3 / regret matching) |
| 6 Heterogeneous players | Same, action space explodes | Shapley DP for small P; SM-MCTS beyond | PSRO over `src/strategies/` |
| 7 Switching constraints | Same + previous-allocation state | SM-MCTS | PSRO with MAPPO best response |
| 8 Commander delay | Same + command queue in state | Shapley DP / SM-MCTS (queue in state) | PSRO |
| 9 Partial information | Imperfect info, zero-sum | Tabular CFR / MCCFR (small); POMCP vs fixed opponent | Deep CFR |
| 10 Spies | Imperfect info + chance | CFR with spy assignment as chance node | Deep CFR; POMCP baseline |
| 11 Full realistic | Large imperfect-info stochastic game | ReBeL-style (belief + search + learned value) | PSRO + MAPPO/QMIX; AlphaZero with simultaneous-move nodes |

Rule of thumb: **fix the per-turn solve first**, replacing `max min` with a matrix-game value. Only then scale up with sampling (SM-MCTS) and learning (PSRO, Deep CFR, ReBeL).

---

## Why Sequential Maximin ≠ Simultaneous-Move Value

### Matching pennies (canonical 2×2)

A's payoff (B receives the negative):

```text
            B: H    B: T
A: H        +1      -1
A: T        -1      +1
```

| Solution concept | Who commits first | Value for A |
|---|---|---|
| Pure maximin `max_a min_b` | A commits, B best-responds | **−1** |
| Pure minimax `min_b max_a` | B commits, A best-responds | **+1** |
| Mixed (Shapley / von Neumann) value | Simultaneous | **0**, with A = (½, ½) and B = (½, ½) |

The current solver returns −1. That is the value of a game in which B is allowed to see A's choice, which is not the game we simulate. The true value is 0. With mixed strategies the lower and upper values coincide (the minimax theorem). With pure strategies they don't.

### The same gap in a tug-of-war turn

One turn, 2 ropes. Team A has 3 players and Team B has 2. Each rope contributes `sign(a_i − b_i)`, so A's payoff is ropes won minus ropes lost:

```text
             B:(2,0)  B:(1,1)  B:(0,2)
A:(3,0)        1        0        0
A:(2,1)        1        1        0
A:(1,2)        0        1        1
A:(0,3)        0        0        1
```

- Pure maximin (current `dp.rs` logic): every row's minimum is 0, so **0**.
- Pure minimax: every column's maximum is 1, so **1**.
- Mixed value: **0.5**. A plays (2,1) and (1,2) with probability ½ each; B plays (2,0) and (0,2) with probability ½ each. Both guarantees equal 0.5. (Checked numerically by grid search over A's mixtures.)

The sequential solver undervalues A by 0.5 per turn here. Over a multi-turn horizon this compounds, and the policy it outputs (any deterministic allocation) is exploitable: a B that predicts it holds A to 0. This is the Blotto structure of Section 1 in miniature, and it is why Section 2 (one LP per state) is the recommended first change.

---

## Key Insights

1. The current DP is correct for the **sequential** game and gives a valid lower bound for A in the simultaneous game. It is not the simultaneous-move value.
2. Shapley (1953) is the minimal correct fix for all fully observed levels. It keeps the existing state space and changes only the per-state operator.
3. SM-MCTS and PSRO cover the jump in size. CFR-family methods cover the jump to hidden information (Levels 9–10).
4. Learning-based methods (AlphaZero/MuZero, QMIX/MAPPO) only give game-theoretic guarantees when they are embedded in an equilibrium-seeking loop (simultaneous-move search, PSRO, or ReBeL).
