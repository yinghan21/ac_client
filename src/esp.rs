use imgui::Ui;
use crate::data::Snapshot;
use crate::memory::{self};
use std::sync::{Arc, RwLock};

pub fn draw(ui: &Ui, snapshot: &Snapshot) {
    let Some((screen_w, screen_h)) = memory::game_client_size() else {
        return;
    };
    let line_origin = [screen_w * 0.5, 0.0];
    for p in &snapshot.players {
        if !p.alive {
            continue;
        }
        let Some(head) = memory::world_to_screen(p.head, &snapshot.view_matrix, screen_w, screen_h) else { continue; };
        let Some(feet) = memory::world_to_screen(p.feet, &snapshot.view_matrix, screen_w, screen_h) else { continue; };
        let height = (feet.y - head.y).abs();
        if height < 2.0 {
            continue;
        }
        let width = height * 0.45;
        let box_min = [head.x - width * 0.5, head.y.min(feet.y)];
        let box_max = [head.x + width * 0.5, head.y.max(feet.y)];
        ui.get_foreground_draw_list()
            .add_line(line_origin, head.to_array(), [1.0, 1.0, 1.0, 1.0])
            .thickness(0.5)
            .build();
        ui.get_foreground_draw_list()
            .add_rect(box_min, box_max, [1.0, 1.0, 1.0, 1.0])
            .thickness(0.5)
            .build();
        if let Some(hp) = p.health {
            if hp > 0 {
                let bar_w = 4.0;
                let bar_x = box_min[0] - 3.0 - bar_w;
                let bar_top = box_min[1];
                let bar_bottom = box_max[1];
                let bar_h = bar_bottom - bar_top;
                if bar_h > 2.0 {
                    let ratio = (hp as f32 / 100.0).clamp(0.0, 1.0);
                    let fill_top = bar_bottom - bar_h * ratio;
                    // 底色：整条，画在血量下面
                    ui.get_foreground_draw_list()
                        .add_rect([bar_x, bar_top], [bar_x + bar_w, bar_bottom], [0.0, 0.0, 0.0, 0.6])
                        .filled(true)
                        .build();
                    // 血量：从底部往上长，满血绿 → 半血黄 → 残血红
                    let tint = if ratio > 0.5 {
                        [(1.0 - ratio) * 2.0, 1.0, 0.0, 1.0]
                    } else {
                        [1.0, ratio * 2.0, 0.0, 1.0]
                    };
                    ui.get_foreground_draw_list()
                        .add_rect([bar_x, fill_top], [bar_x + bar_w, bar_bottom], tint)
                        .filled(true)
                        .build();
                }
            }
        }
    }
}