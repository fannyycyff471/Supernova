use crate::prelude::*;

/// 主菜单输入系统
/// 处理主菜单和制作人员名单界面的按钮选择逻辑
pub fn main_menu_input_system(
    app_state: ResMut<State<AppState>>,                      // 当前 App 状态
    mut next_app_state: ResMut<NextState<AppState>>,        // 下一个 App 状态
    menu_action_state: Res<ActionState<MenuAction>>,        // 菜单动作状态（是否按下了按钮）
    mut app_exit_events: EventWriter<AppExit>,              // 写入退出事件
    menu: Query<&MenuHandler>,                              // 查询菜单选项状态（选中的 ID）
) {
    // 尝试获取菜单（应该只有一个菜单实体）
    if let Ok(menu) = menu.single() {
        // 如果用户按下了确认键（Accept）
        if menu_action_state.just_pressed(&MenuAction::Accept) {
            // 当前处于主菜单状态
            if app_state.get() == &AppState::Menu {
                match menu.selected_id {
                    0 => {
                        // 选项 0：进入游戏
                        next_app_state.set(AppState::Game);
                    }
                    1 => {
                        // 选项 1：查看制作人员名单
                        next_app_state.set(AppState::Credits);
                    }
                    _ => {
                        // 其他选项：退出程序
                        app_exit_events.write(AppExit::Success);
                    }
                }
            }
            // 当前处于制作人员名单界面
            if app_state.get() == &AppState::Credits {
                match menu.selected_id {
                    0 => {
                        // 选项 0：返回主菜单
                        next_app_state.set(AppState::Menu);
                    }
                    _ => {
                        // 其他选项：退出程序
                        app_exit_events.write(AppExit::Success);
                    }
                }
            }
        }
    }
}

/// 游戏内菜单输入系统
/// 处理暂停菜单、游戏结束菜单的按钮选择逻辑
pub fn game_menu_input_system(
    game_state: ResMut<State<GameState>>,                   // 当前游戏状态（运行/暂停/结束）
    mut next_app_state: ResMut<NextState<AppState>>,        // 下一个 App 状态
    mut next_game_state: ResMut<NextState<GameState>>,      // 下一个游戏状态
    menu_action_state: Res<ActionState<MenuAction>>,        // 菜单动作状态
    mut app_exit_events: EventWriter<AppExit>,              // 写入退出事件
    menu: Query<&MenuHandler>,                              // 查询菜单选项状态
) {
    // 按下暂停/恢复按钮
    if menu_action_state.just_pressed(&MenuAction::PauseUnpause) {
        if game_state.get() == &GameState::Running {
            // 当前为运行状态，转为暂停
            next_game_state.set(GameState::Paused);
        }
        if game_state.get() == &GameState::Paused {
            // 当前为暂停状态，转为运行
            next_game_state.set(GameState::Running);
        }
    }

    // 如果存在菜单项
    if let Ok(menu) = menu.single() {
        // 按下确认键
        if menu_action_state.just_pressed(&MenuAction::Accept) {
            // 如果当前是暂停状态的菜单
            if game_state.get() == &GameState::Paused {
                match menu.selected_id {
                    0 => {
                        // 选项 0：继续游戏
                        next_game_state.set(GameState::Running);
                    }
                    1 => {
                        // 选项 1：返回主菜单
                        next_app_state.set(AppState::Menu);
                    }
                    _ => {
                        // 其他：退出程序
                        app_exit_events.write(AppExit::Success);
                    }
                }
            }
            // 如果当前是游戏结束界面
            if game_state.get() == &GameState::Over {
                match menu.selected_id {
                    0 => {
                        // 选项 0：返回主菜单
                        next_app_state.set(AppState::Menu);
                    }
                    _ => {
                        // 其他：退出程序
                        app_exit_events.write(AppExit::Success);
                    }
                }
            }
        }
    }
}
