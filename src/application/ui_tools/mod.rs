use imgui::*;
use gm_unleashed_md::{ *, Style };
use super::{ Fonts, FontStyle };

pub struct Button {
    label : ImString,
    pressed : bool,
}

pub struct TextField {
    label : ImString,
    content : ImString,
}

impl TextField {
    pub fn new(label : ImString) -> Self {
        TextField {
            label,
            content : ImString::with_capacity(1000),
        }
    }

    pub fn build_gui(&mut self, ui : &Ui) {
        ui.input_text(&self.label, &mut self.content).build();
    }

    pub fn content(&self) -> &ImStr { &self.content }
}

impl Button {
    pub fn new(label : ImString) -> Self {
        Button {
            label,
            pressed : false,
        }
    }

    pub fn build_gui(&mut self, ui : &Ui) {
        self.pressed = button(ui, &self.label);
    }

    pub fn pressed(&self) -> bool { self.pressed }
}

pub fn button(ui : &Ui, label : &ImStr) -> bool {
    let text_as_str : &str = label.as_ref();
    let size = [text_as_str.len() as f32 * 10.0, 20.0];
    ui.button(label, size)
}

struct ActiveStyle {
    end : usize,
    style : Style,
}

pub fn markdown<S>(ui : &Ui, raw_md : S, fonts : &Fonts)
    where S : Into<String> 
{
    let [offset, _] = ui.cursor_pos();
    let md = parse(tokenize(raw_md.into()));
    let mut breaks = md.breaks.into_iter().peekable();
    let mut styles = md.styles.into_iter().peekable();
    let mut active_font_style = FontStyle::Normal;
    let outer_font = ui.push_font(*fonts.get(&FontStyle::Normal));
    let mut active_styles = Vec::new();
    for (idx, text) in md.text.into_iter().map(|s| {ImString::new(s)}).enumerate() {
        while let Some(style) = styles.peek() {
            if style.span.start == idx {
                let style = styles.next().unwrap();
                active_font_style = active_font_style + style.style.clone();
                active_styles.push(ActiveStyle {
                    end : style.span.end,
                    style : style.style,
                });
            } else {
                break
            }
        }
        let font = ui.push_font(*fonts.get(&active_font_style));
        wrapped_text(ui, &text, offset);
        font.pop(ui);
        let mut new_active_styles = Vec::new();
        for active_style in active_styles {
            if active_style.end == idx {
                active_font_style = active_font_style - active_style.style;
            } else {
                new_active_styles.push(active_style);
            }
        }
        active_styles = new_active_styles;
        if Some(&Break{ pos : idx + 1 } ) == breaks.peek() {
            ui.same_line(offset); 
            let mut new_pos = ui.cursor_pos();
            new_pos[1] += ui.current_font_size();
            ui.set_cursor_pos(new_pos);
            breaks.next();
            while Some(&Break{ pos : idx + 1 } ) == breaks.peek() {
                breaks.next();
                new_pos[1] += ui.current_font_size();
                ui.set_cursor_pos(new_pos);
            }
        } else {
            ui.same_line(ui.item_rect_max()[0] - ui.window_pos()[0]); 
        }
    }
    outer_font.pop(ui);
    ui.new_line();
}

pub fn wrapped_text(ui : &Ui, text : &ImString, line_start : f32) {
    let [max_x ,_] = ui.window_size();
    let [offset, start_y] = ui.cursor_pos();
    let wrap_pos  = unsafe { 
        use imgui::internal::RawCast;
        imgui::sys::ImFont_CalcWordWrapPositionA(
            ui.current_font().raw() as *const _ as *mut _, 
            1.0, 
            text.as_ptr(), 
            text.as_ptr().add(text.to_str().len()), 
            max_x - offset
        ) as usize - text.as_ptr() as usize
    };
    let first_line = ImString::new(&text.to_str()[..wrap_pos as usize]);
    if ui.calc_text_size(&first_line, false, 0.0)[0] <= max_x - offset {
        ui.text(&first_line);
    } else {
        ui.set_cursor_pos([line_start, start_y + ui.text_line_height_with_spacing()]);
        ui.text_wrapped(&first_line);
    }
    ui.set_cursor_pos([line_start, start_y + ui.text_line_height_with_spacing()]);
    let mut rest = &text.to_str()[wrap_pos as usize..];
    while rest.chars().nth(0) == Some(' ') {
        rest = &rest[1..];
    }
    if rest != "" {
        ui.text_wrapped(&ImString::new(rest));
    }
}