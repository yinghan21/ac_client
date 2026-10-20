mod offset;
mod memory;
mod data;
mod esp;
use crate::data::Snapshot;
use std::ffi::c_void;
use std::sync::{Arc, RwLock};
use hudhook::{Hudhook, ImguiRenderLoop};
use hudhook::hooks::opengl3::ImguiOpenGl3Hooks;
use hudhook::windows::Win32::Foundation::HINSTANCE;
use hudhook::windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use imgui::Ui;
use imgui::*;

struct EspLoop {
    snapshot: Arc<RwLock<Snapshot>>,
}
impl ImguiRenderLoop for EspLoop {
    fn render(&mut self, ui: &mut Ui) {
        let snapshot = match self.snapshot.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        esp::draw(ui, &snapshot);
    }
}

fn run(module_raw: usize) -> anyhow::Result<()> {
    std::thread::sleep(std::time::Duration::from_millis(2));
    let module = module_raw as *mut c_void;
    let module_base = memory::module_base().ok_or_else(|| anyhow::anyhow!("get module_base err"))?;
    let snapshot = Arc::new(RwLock::new(Snapshot::default()));
    data::spawn(module_base, Arc::clone(&snapshot));
    let render_loop = EspLoop {
        snapshot,
    };
    Hudhook::builder()
        .with::<ImguiOpenGl3Hooks>(render_loop)
        .with_hmodule(HINSTANCE(module))
        .build()
        .apply()
        .map_err(|e| anyhow::anyhow!("hudhook error: {e:?}"))?;
    Ok(())
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "system" fn DllMain(module: *mut c_void, call_reason: u32, _: *mut c_void) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        let module_raw = module as usize;
        std::thread::spawn(move || {
            if let Err(err) = run(module_raw) {
                eprintln!("[ac-esp] initialize error: {err:?}");
            }
        });
    }
    true
}