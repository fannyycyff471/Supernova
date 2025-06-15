Supernova 项目说明文档
项目介绍
Supernova 是一个基于 Rust 和 Bevy 引擎开发的射击类游戏项目，融合了现代游戏技术与丰富的特效，旨在提供流畅、视觉震撼的游戏体验。该项目结构清晰，模块划分合理，便于扩展和维护。
运行说明
1. 环境要求：Rust 1.70 以上，支持 Bevy 0.16，需安装依赖库。
2. 依赖安装：运行 cargo build 会自动拉取所有依赖。
3. 启动项目：使用命令 cargo run 来编译并运行游戏。
4. 配置文件：无特殊配置，所有参数均硬编码或在代码中定义。
依赖说明
- bevy 0.16: 游戏引擎核心。
- bevy_hanabi: 粒子特效插件。
- leafwing-input-manager: 输入管理。
- rand: 随机数生成。
- avian2d: 2D工具库。
- 其它依赖请见 Cargo.toml。

贡献指南
欢迎提交 Pull Request 和 Issues。
请遵循项目的代码规范，保持代码整洁，详细说明变更内容。
所有提交需遵守 MIT 或 Apache-2.0 许可证。
许可证
本项目采用 MIT 或 Apache-2.0 双许可证授权，详见 LICENSE 文件。
致谢
特别感谢 Kenney Vleugels 提供的游戏素材，Pablo Roman Andrioli 贡献的着色器灵感，以及所有开源社区的支持。
