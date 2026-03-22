<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.md">English</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/saints-mile/readme.png" width="400" alt="Saint's Mile" />
</p>

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/actions"><img src="https://github.com/mcp-tool-shop-org/saints-mile/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://mcp-tool-shop-org.github.io/saints-mile/"><img src="https://img.shields.io/badge/Landing_Page-saints--mile-d97706" alt="Landing Page"></a>
</p>

一款面向那些曾经喜爱此类游戏的成年人的前沿JRPG。

《圣徒之路》是一款回合制战术角色扮演游戏，背景设定在“灰烬盆地”——一个正在被铁路、水路和法律重塑的边境地区。你将扮演加伦·鲁克，一个名声先于他的人，在被他人撰写的通缉令下，度过近四十年的人生。

该游戏使用Rust语言构建，专为终端环境设计。没有过多的图形元素，专注于确定性的机制、战术战斗以及一个信任玩家的剧情。

## 游戏特色

- 一款**90年代风格的JRPG**，拥有4个角色组成的队伍，独特的角色定位，双人技能，以及回合制战斗。
- 一款**西部题材**的游戏，声誉如同蛛网般复杂，距离会影响决策，而道路本身就是一个迷宫。
- 一款**面向成年人**的游戏，探讨着遗憾、责任、妥协、衰老、忠诚以及重新开始等主题。
- 一款**原生终端体验**，通过[ratatui](https://ratatui.rs/)可以在任何支持终端的设备上运行。

## 游戏剧情

游戏时间跨度近四十年：从一个十九岁的、仍然认为法律和真相是相关联的警卫，到一个背负他人罪行的年轻枪手，到一个成熟的亡命之徒，带着一群身怀伤痕的专家穿越荒凉的盆地，到一个年老的男人，被迫决定一个人的一生是否可以通过行为、通过真相，或者根本无法得到救赎。

表面的冲突是铁路、水路和土地的争夺。更深层次的冲突是，谁有权书写发生在“圣徒之路”的故事。

## 战斗

在每一次重要的战斗开始时，都会出现紧张的对峙——双手悬在空中，神经受到考验，主动权需要争取。然后，一个完整的战术角色扮演游戏战斗系统展开：从六个角色中选择四个进行战斗，每个角色都有独特的指令，通过剧情和羁绊可以提升技能，还可以使用双人技能，从而获得团队投资的回报。

西部元素改变了游戏的机制，而不仅仅是风格：使用弹药代替MP，使用神经代替士气，使用韧性代替防御加成，战斗之间会留下伤痕。

## 队伍

| 角色 | 定位 | 战斗身份 |
|-----------|------|----------------|
| **Galen Rook** | 枪手 | 精准射击，精确瞄准，战场指挥。随着年龄的增长而进化。 |
| **Eli Winter** | 骗子 | 神经攻击，干扰，使用卑鄙的手段。忠诚度在后期解锁。 |
| **Dr. Ada Mercer** | 医生 | 治疗，伤口处理，揭示弱点。 |
| **Rosa Varela** | 牧场工人 | 使用套索控制敌人，作为前线坦克，施加位置压力。 |
| **Rev. Miriam Slate** | 牧师 | 提供增益效果，支持神经，控制人群。 |
| **Lucien "Fuse" Marr** | 炸弹专家 | 延迟范围攻击，破坏环境，改变地形。 |

## 状态

**第一阶段 — 核心开发。** 完整的剧情已设计完成（序章 + 15个章节）。 构建核心功能和运行时合约已锁定。 接下来是实现开场部分。

## 安全模型

《圣徒之路》是一款单人离线游戏。它不具备以下功能：
- 连接互联网
- 收集遥测数据或分析数据
- 访问其自身保存目录之外的文件
- 需要任何超出终端输入/输出的权限

存档文件以RON格式存储在用户可访问的目录中。

## 系统要求

- Rust 1.75+ (2021版本)
- 任何支持256色显示的终端

## 许可证

MIT

---

由 <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a> 构建。
