extern crate base64;
extern crate console_error_panic_hook;
extern crate console_log;
extern crate log;
extern crate rand;
extern crate serde_json;
extern crate bitflags;
extern crate unicode_segmentation;
extern crate wasm_bindgen;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate enum_primitive;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis, js_name = __encrusted_js_message)]
    fn js_message(mtype: &str, message: &str);

    #[wasm_bindgen(js_namespace = globalThis, js_name = __encrusted_rand)]
    fn rand() -> u32;
}

mod buffer;
mod frame;
mod instruction;
mod options;
mod quetzal;
mod traits;
mod ui_web;
mod zmachine;
mod chgrid;

use options::Options;
use ui_web::WebUI;
use zmachine::Zmachine;

#[wasm_bindgen]
pub fn hook() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);
}

fn push_updates(zvm: &mut Zmachine) {
    let map = serde_json::to_string(&zvm.get_current_room()).unwrap();
    let tree = serde_json::to_string(&zvm.get_object_tree()).unwrap();

    zvm.update_status_bar();
    zvm.ui.message("map", &map);
    zvm.ui.message("tree", &tree);

    if zvm.options.log_instructions {
        zvm.ui.message("instructions", &zvm.instr_log);
        zvm.instr_log.clear();
    }
}

#[wasm_bindgen]
pub struct Engine {
    zvm: Zmachine,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(file: &[u8]) -> Engine {
        hook();
        let ui = WebUI::new();
        let mut opts = Options::default();
        opts.rand_seed = [rand(), rand(), rand(), rand()];

        Engine {
            zvm: Zmachine::new(file.to_vec(), ui, opts),
        }
    }

    pub fn step(&mut self) -> bool {
        let done = self.zvm.step();

        self.zvm.ui.flush();
        push_updates(&mut self.zvm);
        done
    }

    pub fn feed(&mut self, input: &str) {
        self.zvm.handle_input(input.to_owned());
    }

    pub fn restore(&mut self, b64: &str) {
        self.zvm.restore(b64);
    }

    pub fn load_savestate(&mut self, b64: &str) {
        self.zvm.load_savestate(b64);
    }

    pub fn get_updates(&mut self) {
        push_updates(&mut self.zvm);
    }

    pub fn undo(&mut self) -> bool {
        self.zvm.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.zvm.redo()
    }

    pub fn enable_instruction_logs(&mut self, enabled: bool) {
        self.zvm.options.log_instructions = enabled;
    }

    pub fn get_object_details(&self, obj_num: u16) -> String {
        self.zvm.debug_object_details(obj_num)
    }

    pub fn flush_log(&self) {
        self.zvm.ui.message("instructions", &self.zvm.instr_log);
    }

    pub fn set_terp_caps(&mut self, terp_caps_json: &str) {
        let v: serde_json::Value = serde_json::from_str(terp_caps_json)
            .expect("Incorrect interpreter capabilities");
        self.zvm.set_terp_caps(v);
    }
}
