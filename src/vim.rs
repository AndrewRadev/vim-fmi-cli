use std::fs;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::process::Command;

use anyhow::anyhow;
use once_cell::sync::OnceCell;
use which::which;

pub struct Vim {
    pub executable: String,
    vimrc_path: PathBuf,
}

impl Vim {
    pub fn new(vimrc_path: PathBuf) -> ::anyhow::Result<Self> {
        let executable =
            if let Ok(custom_value) = std::env::var("VIMFMI_EXECUTABLE") {
                custom_value
            } else if which("mvim").is_ok() {
                String::from("mvim")
            } else if which("gvim").is_ok() {
                String::from("gvim")
            } else if which("vim").is_ok() {
                String::from("vim")
            } else {
                return Err(anyhow!("Не беше намерен нито `mvim`, нито `gvim`, нито `vim`, вижте дали програмата е в $PATH"));
            };

        Ok(Self { executable, vimrc_path })
    }

    pub fn run(&self, input_path: &Path, log_path: &Path) -> ::anyhow::Result<(String, Vec<u8>)> {
        // -Z         - restricted mode, utilities not allowed
        // -n         - no swap file, memory only editing
        // --noplugin - don't load any plugins, lets be fair!
        // --nofork   - otherwise gvim forks and returns immediately, mvim has permission issues
        // -i NONE    - don't load .viminfo (for saved macros and the like)
        // +0         - always start on line 0
        // -u vimrc   - load vimgolf .vimrc to level the playing field
        // -U NONE    - don't load .gvimrc
        // -W logfile - keylog file (overwrites if already exists)
        let mut command = Command::new(&self.executable);
        let mut command = &mut command;

        if self.executable != "nvim" {
            command = command.args(["--nofork", "-Z"]);
        }

        command = command.
            args(["-n", "--noplugin", "-i", "NONE", "+0", "-U", "NONE"]).
            args(["-u", self.vimrc_path.to_str().unwrap()]).
            args(["-W", log_path.to_str().unwrap()]).
            arg(input_path.to_str().unwrap());

        if !command.spawn()?.wait()?.success() {
            return Err(anyhow!("Vim излезе с неуспешен статус."));
        }

        let result = fs::read_to_string(input_path)?;
        let log = fs::read(log_path)?;

        Ok((result, log))
    }
}

#[derive(Debug)]
pub struct Keylog {
    bytes: Vec<u8>,
}

impl Keylog {
    pub fn new(bytes: &[u8]) -> Self {
        let bytes = bytes.to_owned();
        Keylog { bytes }
    }

    pub fn into_iter(&self) -> impl Iterator<Item = String> + '_ {
        let mut bytes_iter = self.bytes.iter();
        let byte_translation = kc_1byte();
        let mbyte_translation = kc_mbyte();

        ::std::iter::from_fn(move || {
            let byte = *bytes_iter.next()?;

            if byte == 0x80 {
                let mbytes = vec![
                    *bytes_iter.next()?,
                    *bytes_iter.next()?,
                ];
                let result = mbyte_translation.get(&mbytes).
                    map(String::clone).
                    unwrap_or_else(String::new);
                Some(result)
            } else {
                Some(byte_translation[byte as usize].clone())
            }
        })
    }
}

fn kc_1byte() -> &'static Vec<String> {
    static INSTANCE: OnceCell<Vec<String>> = OnceCell::new();

    INSTANCE.get_or_init(|| {
        let mut data = Vec::with_capacity(256);

        // (0..255).each {|n| KC_1BYTE.push("<%#04x>" % n)} # Fallback for non-ASCII
        for index in 0..=255 {
            data.push(format!("<{:#04x}>", index));
        }

        // (1..127).each {|n| KC_1BYTE[n] = "<C-#{(n ^ 0x40).chr}>"}
        for index in 1..=127 {
            data[index as usize] = format!("<C-{}>", char::from_u32(index ^ 0x40).unwrap());
        }

        // (32..126).each {|c| KC_1BYTE[c] = c.chr } # Printing chars
        for index in 32..=126 {
            data[index as usize] = char::from_u32(index).unwrap().to_string();
        }

        // KC_1BYTE[0x1b] = "<Esc>" # Special names for a few control chars
        // KC_1BYTE[0x0d] = "<CR>"
        // KC_1BYTE[0x0a] = "<NL>"
        // KC_1BYTE[0x09] = "<Tab>"
        data[0x1b] = String::from("<Esc>");
        data[0x0d] = String::from("<CR>");
        data[0x0a] = String::from("<NL>");
        data[0x09] = String::from("<Tab>");

        data
    })
}

fn kc_mbyte() -> &'static HashMap<Vec<u8>, String> {
    static INSTANCE: OnceCell<HashMap<Vec<u8>, String>> = OnceCell::new();

    INSTANCE.get_or_init(|| {
        let mut data = HashMap::new();

        // This list has been populated by looking at
        // :h terminal-options and vim source files:
        // keymap.h and misc2.c
        data.insert(b"k1".to_vec(), String::from("<F1>"));
        data.insert(b"k2".to_vec(), String::from("<F2>"));
        data.insert(b"k3".to_vec(), String::from("<F3>"));
        data.insert(b"k4".to_vec(), String::from("<F4>"));
        data.insert(b"k5".to_vec(), String::from("<F5>"));
        data.insert(b"k6".to_vec(), String::from("<F6>"));
        data.insert(b"k7".to_vec(), String::from("<F7>"));
        data.insert(b"k8".to_vec(), String::from("<F8>"));
        data.insert(b"k9".to_vec(), String::from("<F9>"));
        data.insert(b"k;".to_vec(), String::from("<F10>"));
        data.insert(b"F1".to_vec(), String::from("<F11>"));
        data.insert(b"F2".to_vec(), String::from("<F12>"));
        data.insert(b"F3".to_vec(), String::from("<F13>"));
        data.insert(b"F4".to_vec(), String::from("<F14>"));
        data.insert(b"F5".to_vec(), String::from("<F15>"));
        data.insert(b"F6".to_vec(), String::from("<F16>"));
        data.insert(b"F7".to_vec(), String::from("<F17>"));
        data.insert(b"F8".to_vec(), String::from("<F18>"));
        data.insert(b"F9".to_vec(), String::from("<F19>"));

        data.insert(b"%1".to_vec(), String::from("<Help>"));
        data.insert(b"&8".to_vec(), String::from("<Undo>"));
        data.insert(b"#2".to_vec(), String::from("<S-Home>"));
        data.insert(b"*7".to_vec(), String::from("<S-End>"));
        data.insert(b"K1".to_vec(), String::from("<kHome>"));
        data.insert(b"K4".to_vec(), String::from("<kEnd>"));
        data.insert(b"K3".to_vec(), String::from("<kPageUp>"));
        data.insert(b"K5".to_vec(), String::from("<kPageDown>"));
        data.insert(b"K6".to_vec(), String::from("<kPlus>"));
        data.insert(b"K7".to_vec(), String::from("<kMinus>"));
        data.insert(b"K8".to_vec(), String::from("<kDivide>"));
        data.insert(b"K9".to_vec(), String::from("<kMultiply>"));
        data.insert(b"KA".to_vec(), String::from("<kEnter>"));
        data.insert(b"KB".to_vec(), String::from("<kPoint>"));
        data.insert(b"KC".to_vec(), String::from("<k0>"));
        data.insert(b"KD".to_vec(), String::from("<k1>"));
        data.insert(b"KE".to_vec(), String::from("<k2>"));
        data.insert(b"KF".to_vec(), String::from("<k3>"));
        data.insert(b"KG".to_vec(), String::from("<k4>"));
        data.insert(b"KH".to_vec(), String::from("<k5>"));
        data.insert(b"KI".to_vec(), String::from("<k6>"));
        data.insert(b"KJ".to_vec(), String::from("<k7>"));
        data.insert(b"KK".to_vec(), String::from("<k8>"));
        data.insert(b"KL".to_vec(), String::from("<k9>"));

        data.insert(b"kP".to_vec(), String::from("<PageUp>"));
        data.insert(b"kN".to_vec(), String::from("<PageDown>"));
        data.insert(b"kh".to_vec(), String::from("<Home>"));
        data.insert(b"@7".to_vec(), String::from("<End>"));
        data.insert(b"kI".to_vec(), String::from("<Insert>"));
        data.insert(b"kD".to_vec(), String::from("<Del>"));
        data.insert(b"kb".to_vec(), String::from("<BS>"));

        data.insert(b"ku".to_vec(), String::from("<Up>"));
        data.insert(b"kd".to_vec(), String::from("<Down>"));
        data.insert(b"kl".to_vec(), String::from("<Left>"));
        data.insert(b"kr".to_vec(), String::from("<Right>"));
        data.insert(b"#4".to_vec(), String::from("<S-Left>"));
        data.insert(b"%i".to_vec(), String::from("<S-Right>"));

        data.insert(b"kB".to_vec(), String::from("<S-Tab>"));
        data.insert(b"\xffX".to_vec(), String::from("<C-@>"));

        // This is how you escape literal 0x80
        data.insert(b"\xfeX".to_vec(), String::from("<0x80>"));

        // These rarely-used modifiers should be combined with the next
        // stroke (like <S-Space>), but let's put them here for now
        data.insert(b"\xfc\x02".to_vec(), String::from("<S->"));
        data.insert(b"\xfc\x04".to_vec(), String::from("<C->"));
        data.insert(b"\xfc\x06".to_vec(), String::from("<C-S->"));
        data.insert(b"\xfc\x08".to_vec(), String::from("<A->"));
        data.insert(b"\xfc\x0a".to_vec(), String::from("<A-S->"));
        data.insert(b"\xfc\x0c".to_vec(), String::from("<C-A>"));
        data.insert(b"\xfc\x0e".to_vec(), String::from("<C-A-S->"));
        data.insert(b"\xfc\x10".to_vec(), String::from("<M->"));
        data.insert(b"\xfc\x12".to_vec(), String::from("<M-S->"));
        data.insert(b"\xfc\x14".to_vec(), String::from("<M-C->"));
        data.insert(b"\xfc\x16".to_vec(), String::from("<M-C-S->"));
        data.insert(b"\xfc\x18".to_vec(), String::from("<M-A->"));
        data.insert(b"\xfc\x1a".to_vec(), String::from("<M-A-S->"));
        data.insert(b"\xfc\x1c".to_vec(), String::from("<M-C-A>"));
        data.insert(b"\xfc\x1e".to_vec(), String::from("<M-C-A-S->"));

        // KS_EXTRA keycodes (starting with 0x80 0xfd) are defined by an enum in
        // Vim's keymap.h. Sometimes, a new Vim adds or removes a keycode, which
        // changes the binary representation of every keycode after it. Very
        // annoying.
        data.insert(b"\xfd\x04".to_vec(), String::from("<S-Up>"));
        data.insert(b"\xfd\x05".to_vec(), String::from("<S-Down>"));
        data.insert(b"\xfd\x06".to_vec(), String::from("<S-F1>"));
        data.insert(b"\xfd\x07".to_vec(), String::from("<S-F2>"));
        data.insert(b"\xfd\x08".to_vec(), String::from("<S-F3>"));
        data.insert(b"\xfd\x09".to_vec(), String::from("<S-F4>"));
        data.insert(b"\xfd\x0a".to_vec(), String::from("<S-F5>"));
        data.insert(b"\xfd\x0b".to_vec(), String::from("<S-F6>"));
        data.insert(b"\xfd\x0c".to_vec(), String::from("<S-F7>"));
        data.insert(b"\xfd\x0d".to_vec(), String::from("<S-F9>"));
        data.insert(b"\xfd\x0e".to_vec(), String::from("<S-F10>"));
        data.insert(b"\xfd\x0f".to_vec(), String::from("<S-F10>"));
        data.insert(b"\xfd\x10".to_vec(), String::from("<S-F11>"));
        data.insert(b"\xfd\x11".to_vec(), String::from("<S-F12>"));
        data.insert(b"\xfd\x12".to_vec(), String::from("<S-F13>"));
        data.insert(b"\xfd\x13".to_vec(), String::from("<S-F14>"));
        data.insert(b"\xfd\x14".to_vec(), String::from("<S-F15>"));
        data.insert(b"\xfd\x15".to_vec(), String::from("<S-F16>"));
        data.insert(b"\xfd\x16".to_vec(), String::from("<S-F17>"));
        data.insert(b"\xfd\x17".to_vec(), String::from("<S-F18>"));
        data.insert(b"\xfd\x18".to_vec(), String::from("<S-F19>"));
        data.insert(b"\xfd\x19".to_vec(), String::from("<S-F20>"));
        data.insert(b"\xfd\x1a".to_vec(), String::from("<S-F21>"));
        data.insert(b"\xfd\x1b".to_vec(), String::from("<S-F22>"));
        data.insert(b"\xfd\x1c".to_vec(), String::from("<S-F23>"));
        data.insert(b"\xfd\x1d".to_vec(), String::from("<S-F24>"));
        data.insert(b"\xfd\x1e".to_vec(), String::from("<S-F25>"));
        data.insert(b"\xfd\x1f".to_vec(), String::from("<S-F26>"));
        data.insert(b"\xfd\x20".to_vec(), String::from("<S-F27>"));
        data.insert(b"\xfd\x21".to_vec(), String::from("<S-F28>"));
        data.insert(b"\xfd\x22".to_vec(), String::from("<S-F29>"));
        data.insert(b"\xfd\x23".to_vec(), String::from("<S-F30>"));
        data.insert(b"\xfd\x24".to_vec(), String::from("<S-F31>"));
        data.insert(b"\xfd\x25".to_vec(), String::from("<S-F32>"));
        data.insert(b"\xfd\x26".to_vec(), String::from("<S-F33>"));
        data.insert(b"\xfd\x27".to_vec(), String::from("<S-F34>"));
        data.insert(b"\xfd\x28".to_vec(), String::from("<S-F35>"));
        data.insert(b"\xfd\x29".to_vec(), String::from("<S-F36>"));
        data.insert(b"\xfd\x2a".to_vec(), String::from("<S-F37>"));
        data.insert(b"\xfd\x2b".to_vec(), String::from("<Mouse>"));
        data.insert(b"\xfd\x2c".to_vec(), String::from("<LeftMouse>"));
        data.insert(b"\xfd\x2d".to_vec(), String::from("<LeftDrag>"));
        data.insert(b"\xfd\x2e".to_vec(), String::from("<LeftRelease>"));
        data.insert(b"\xfd\x2f".to_vec(), String::from("<MiddleMouse>"));
        data.insert(b"\xfd\x30".to_vec(), String::from("<MiddleDrag>"));
        data.insert(b"\xfd\x31".to_vec(), String::from("<MiddleRelease>"));
        data.insert(b"\xfd\x32".to_vec(), String::from("<RightMouse>"));
        data.insert(b"\xfd\x33".to_vec(), String::from("<RightDrag>"));
        data.insert(b"\xfd\x34".to_vec(), String::from("<RightRelease>"));
        data.insert(b"\xfd\x35".to_vec(), String::from("")); // KE_IGNORE
        //"\xfd\x36" => "KE_TAB",
        //"\xfd\x37" => "KE_S_TAB_OLD",

        // Vim 7.4.1433 removed KE_SNIFF. Unfortunately, this changed the
        // offset of every keycode after it.
        // Vim 8.0.0697 added back a KE_SNIFF_UNUSED to fill in for the
        // removed KE_SNIFF.
        // Keycodes after this point should be accurate for vim < 7.4.1433
        // and vim > 8.0.0697.
        //"\xfd\x38" => "KE_SNIFF",
        //"\xfd\x39" => "KE_XF1",
        //"\xfd\x3a" => "KE_XF2",
        //"\xfd\x3b" => ".to_vec()KE_XF3",
        //"\xfd\x3c" => "KE_XF4",
        //"\xfd\x3d" => "KE_XEND",
        //"\xfd\x3e" => "KE_ZEND",
        //"\xfd\x3f" => "KE_XHOME",
        //"\xfd\x40" => "KE_ZHOME",
        //"\xfd\x41" => "KE_XUP",
        //"\xfd\x42" => "KE_XDOWN",
        //"\xfd\x43" => "KE_XLEFT",
        //"\xfd\x44" => "KE_XRIGHT",
        //"\xfd\x45" => "KE_LEFTMOUSE_NM",
        //"\xfd\x46" => "KE_LEFTRELEASE_NM",
        //"\xfd\x47" => "KE_S_XF1",
        //"\xfd\x48" => "KE_S_XF2",
        //"\xfd\x49" => "KE_S_XF3",
        //"\xfd\x4a" => "KE_S_XF4",
        data.insert(b"\xfd\x4b".to_vec(), String::from("<ScrollWheelUp>"));
        data.insert(b"\xfd\x4c".to_vec(), String::from("<ScrollWheelDown>"));

        // Horizontal scroll wheel support was added in Vim 7.3c. These
        // 2 entries shifted the rest of the KS_EXTRA mappings down 2.
        // Though Vim 7.2 is rare today, it was common soon after
        // vimgolf.com was launched. In cases where the 7.3 code is
        // never used but the 7.2 code was common, it makes sense to use
        // the 7.2 code. There are conflicts though, so some legacy
        // keycodes have to stay wrong.
        data.insert(b"\xfd\x4d".to_vec(), String::from("<ScrollWheelRight>"));
        data.insert(b"\xfd\x4e".to_vec(), String::from("<ScrollWheelLeft>"));
        data.insert(b"\xfd\x4f".to_vec(), String::from("<kInsert>"));
        data.insert(b"\xfd\x50".to_vec(), String::from("<kDel>"));
        data.insert(b"\xfd\x51".to_vec(), String::from("<0x9b>")); // :help <CSI>
        //"\xfd\x52" => "KE_SNR",
        //"\xfd\x53" => "KE_PLUG", # never used
        data.insert(b"\xfd\x53".to_vec(), String::from("<C-Left>")); // 7.2 compat
        //"\xfd\x54" => "KE_CMDWIN", # never used
        data.insert(b"\xfd\x54".to_vec(), String::from("<C-Right>")); // 7.2 compat
        data.insert(b"\xfd\x55".to_vec(), String::from("<C-Left>")); // 7.2 <C-Home> conflict
        data.insert(b"\xfd\x56".to_vec(), String::from("<C-Right>")); // 7.2 <C-End> conflict
        data.insert(b"\xfd\x57".to_vec(), String::from("<C-Home>"));
        data.insert(b"\xfd\x58".to_vec(), String::from("<C-End>"));
        //"\xfd\x59" => "KE_X1MOUSE",
        //"\xfd\x5a" => "KE_X1DRAG",
        //"\xfd\x5b" => "KE_X1RELEASE",
        //"\xfd\x5c" => "KE_X2MOUSE",
        //"\xfd\x5d" => "KE_X2DRAG",
        //"\xfd\x5e" => "KE_X2RELEASE",
        data.insert(b"\xfd\x5e".to_vec(), String::from("")); // 7.2 compat (I think?)
        //"\xfd\x5f" => "KE_DROP",
        //"\xfd\x60" => "KE_CURSORHOLD",

        // If you use gvim, you'll get an entry in your keylog every time the
        // window gains or loses focus. These "keystrokes" should not show and
        // should not be counted.
        data.insert(b"\xfd\x60".to_vec(), String::from("")); // 7.2 Focus Gained compat
        data.insert(b"\xfd\x61".to_vec(), String::from("")); // Focus Gained (GVIM) (>7.4.1433)
        data.insert(b"\xfd\x62".to_vec(), String::from("")); // Focus Gained (GVIM)
        data.insert(b"\xfd\x63".to_vec(), String::from("")); // Focus Lost (GVIM)

        data
    })
}
