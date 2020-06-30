use glium::{ 
    *,
    glutin::{ dpi, ContextBuilder, event::*, event_loop::*, window::* } 
};
use imgui_glium_renderer::*;
use imgui_winit_support::*;
use super::*;
use std::collections::*;

#[derive(Clone)]
pub struct Fonts {
    _fonts : HashMap<FontStyle, FontId>,
}

impl Fonts {
    pub fn new(fonts : HashMap<FontStyle, FontId>) -> Result<Self, ()> {
        for font_style in FontStyle::all() {
            if fonts.get(&font_style) == None {
                return Err(())
            }
        }
        Ok(Fonts {
            _fonts : fonts,
        })
    }

    pub fn get(&self, font_style : &FontStyle) -> &FontId {
        self._fonts.get(font_style).unwrap()
    }
}
    

pub struct AppRenderer {
    event_loop : EventLoop<()>,
    platform : WinitPlatform,
    gui : Context,
    display : Display,
    renderer : Renderer,
    fonts : Fonts,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum FontStyle {
    Normal,
    Bold,
    Italic,
    BoldItalic,
}

impl FontStyle {
    pub fn all() -> Vec<FontStyle>
    {
        use FontStyle::*;
        vec![Normal, Bold, Italic, BoldItalic]
    }
}

impl std::ops::Add<gm_unleashed_md::Style> for FontStyle {
    type Output = FontStyle;
    fn add(self, md_style : gm_unleashed_md::Style) -> Self::Output {
        use FontStyle::*;
        match (self, md_style) {
             (Normal, gm_unleashed_md::Style::Italic) => { Italic }
             (Normal, gm_unleashed_md::Style::Bold) => { Bold }
             (Italic, gm_unleashed_md::Style::Bold) => { BoldItalic }
             (Bold, gm_unleashed_md::Style::Italic) => { BoldItalic }
             (s, _) => { s }
        }
    }
}

impl std::ops::Sub<gm_unleashed_md::Style> for FontStyle {
    type Output = FontStyle;
    fn sub(self, md_style : gm_unleashed_md::Style) -> Self::Output {
        use FontStyle::*;
        match (self, md_style) {
            (BoldItalic, gm_unleashed_md::Style::Bold) => { Italic }
            (BoldItalic, gm_unleashed_md::Style::Italic) => { Bold }
            (Italic, gm_unleashed_md::Style::Italic) => { Normal }
            (Bold, gm_unleashed_md::Style::Bold) => { Normal }
            (s, _) => { s }
        }
    }
}

impl AppRenderer {
    pub fn new() -> Self {
        const FONT_SIZE : f32 = 24.0;
        let event_loop = EventLoop::new();
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
          
        let display = Display::new(
            WindowBuilder::new()
                .with_title("GM Unleashed")
                .with_inner_size(dpi::LogicalSize::new(1024, 768))
                .with_transparent(false)
                .with_maximized(true)
                .with_decorations(true),
            ContextBuilder::new().with_vsync(true),
            &event_loop
        ).expect("Error in setting up the display");
        let mut renderer = Renderer::init(&mut imgui, &display).expect("Error in setting up the renderer");
        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &display.gl_window().window(), HiDpiMode::Rounded);

        let mut fonts = HashMap::new();
        imgui.fonts().add_font( &[
            FontSource::DefaultFontData {
                config : Some(FontConfig{
                    size_pixels : 12.0,
                    ..FontConfig::default()
                })
            },
        ]);
        fonts.insert(FontStyle::Normal, imgui.fonts().add_font(&[
            FontSource::TtfData {
                data: include_bytes!("../assets/arial.ttf"),
                size_pixels: FONT_SIZE,
                config: Some(FontConfig {
                    rasterizer_multiply: 1.0,
                    glyph_ranges: FontGlyphRanges::default(),
                    ..FontConfig::default()
                }),
            },
        ]));
        fonts.insert(FontStyle::Italic, imgui.fonts().add_font(&[
            FontSource::TtfData {
                data: include_bytes!("../assets/arial_italic.ttf"),
                size_pixels: FONT_SIZE,
                config: Some(FontConfig {
                    rasterizer_multiply: 1.0,
                    glyph_ranges: FontGlyphRanges::default(),
                    ..FontConfig::default()
                }),
            },
        ]));        
        fonts.insert(FontStyle::Bold, imgui.fonts().add_font(&[
            FontSource::TtfData {
                data: include_bytes!("../assets/arial_bold.ttf"),
                size_pixels: FONT_SIZE,
                config: Some(FontConfig {
                    rasterizer_multiply: 1.0,
                    glyph_ranges: FontGlyphRanges::default(),
                    ..FontConfig::default()
                }),
            },
        ]));       
        fonts.insert(FontStyle::BoldItalic, imgui.fonts().add_font(&[
            FontSource::TtfData {
                data: include_bytes!("../assets/arial_bold_italic.ttf"),
                size_pixels: FONT_SIZE,
                config: Some(FontConfig {
                    rasterizer_multiply: 1.0,
                    glyph_ranges: FontGlyphRanges::default(),
                    ..FontConfig::default()
                }),
            },
        ]));       
        
        renderer.reload_font_texture(&mut imgui).unwrap();
    
        AppRenderer {
            event_loop,
            platform,
            gui : imgui,
            display,
            renderer,
            fonts : Fonts::new(fonts).unwrap(),
        }
    }

    pub fn run<F>(self, mut build_gui : F) 
        where F : FnMut(&mut Ui) + 'static
    {
        let AppRenderer {
            event_loop,
            display,
            mut platform,
            mut gui,
            mut renderer,
            ..
        } = self;
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::MainEventsCleared => {
                    platform.prepare_frame(gui.io_mut(), &display.gl_window().window()).expect("Could not prepare frame.");
                    display.gl_window().window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let mut frame = gui.frame();
                    build_gui(&mut frame);
                    platform.prepare_render(&frame, display.gl_window().window());
                    let mut rendered_frame = display.draw();
                    rendered_frame.clear_color_srgb(0.0, 0.0, 0.0, 0.0);
                    renderer.render(&mut rendered_frame, frame.render()).expect("Could not render.");
                    rendered_frame.finish().expect("Could not swap buffers.");
                },
                Event::WindowEvent{
                    event : WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                },
                event => {
                    platform.handle_event(gui.io_mut(), display.gl_window().window(), &event);
                },
            }
        });
    }

    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }
}