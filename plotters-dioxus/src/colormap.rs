pub enum ColorMap {
    Jet,     // FlowJo default (blue → cyan → yellow → red)
    Viridis, // Perceptually uniform
    Heat,    // Black → red → yellow → white
}

impl ColorMap {
    pub fn get_color(&self, value: f64) -> RGB {
        match self {
            ColorMap::Jet => jet_colormap(value),
            _ => todo!("Other colormaps not implemented yet"),
        }
    }
}

pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

fn jet_colormap(t: f64) -> RGB {
    // Classic FlowJo colormap
    let t = t.clamp(0.0, 1.0);

    let r = (1.5 - 4.0 * (t - 0.75).abs()).clamp(0.0, 1.0);
    let g = (1.5 - 4.0 * (t - 0.5).abs()).clamp(0.0, 1.0);
    let b = (1.5 - 4.0 * (t - 0.25).abs()).clamp(0.0, 1.0);

    RGB {
        r: (r * 255.0) as u8,
        g: (g * 255.0) as u8,
        b: (b * 255.0) as u8,
    }
}
