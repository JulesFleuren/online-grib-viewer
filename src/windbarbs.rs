use serde::Deserialize;
use std::collections::HashMap;
/// This file contains SVG paths for different wind barb representations. It is modified
/// from the original source at https://github.com/qulle/svg-wind-barbs. It has the following license:
///
/// BSD 2-Clause License
///
/// Copyright (c) 2021-present, Qulle
/// All rights reserved.
///
/// Redistribution and use in source and binary forms, with or without
/// modification, are permitted provided that the following conditions are met:
///
/// 1. Redistributions of source code must retain the above copyright notice, this
///    list of conditions and the following disclaimer.
///
/// 2. Redistributions in binary form must reproduce the above copyright notice,
///    this list of conditions and the following disclaimer in the documentation
///    and/or other materials provided with the distribution.
///
/// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
/// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
/// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
/// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
/// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
/// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
/// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
/// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
/// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
/// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum ArrowType {
    PivotTip,
    #[default]
    PivotCenter,
    WindBarb,
}

// transform-origin is set to 125, 125 because the original SVGs are 250x250 and the wind barb is centered there
// The placeholders {TRANSLATE}, {ROTATE}, and {SCALE} should be replaced with actual values when generating the final SVG
static ARROW_PATHS: std::sync::LazyLock<HashMap<&str, &str>> = std::sync::LazyLock::new(|| {
    HashMap::from([
        (
            "arrow-pivot-center",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,135V102 M125,148l7-12.1h-14L125,148z\"/>",
        ),
        (
            "arrow-pivot-tip",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V76 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot0",
            "<path fill=\"#1A232D\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,120c2.762,0,5,2.239,5,5c0,2.762-2.238,5-5,5c-2.761,0-5-2.238-5-5C120,122.239,122.239,120,125,120z\"/><path fill=\"none\" stroke=\"#1A232D\" stroke-width=\"2\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,115c5.523,0,10,4.477,10,10c0,5.523-4.477,10-10,10 c-5.523,0-10-4.477-10-10C115,119.477,119.477,115,125,115z\"/>",
        ),
        (
            "knot2",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V76 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot5",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V76 M125,89l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot10",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V89 M125,89l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot15",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V89 M125,89l14-14 M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot20",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V89 M125,89l14-14 M125,100l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot25",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V79 M125,79l14-14 M125,90l14-14 M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot30",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V79 M125,79l14-14 M125,90l14-14 M125,100l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot35",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V69 M125,69l14-14 M125,80l14-14 M125,90l14-14 M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot40",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V69 M125,69l14-14 M125,80l14-14 M125,90l14-14 M125,100l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot45",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V59 M125,59l14-14 M125,70l14-14 M125,80l14-14 M125,90l14-14 M125,100l7-7 M125,125l7-12.1h-14 L125,125z\"/>",
        ),
        (
            "knot50",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V76 M125,76h14l-14,14V76z M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot55",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V76 M125,76h14l-14,14V76z M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot60",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V76 M125,76h14l-14,14V76z M125,100l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot65",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V66 M125,66h14l-14,14V66z M125,90l14-14 M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot70",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V66 M125,66h14l-14,14V66z M125,90l14-14 M125,100l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot75",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V56 M125,56h14l-14,14V56z M125,80l14-14 M125,90l14-14 M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot80",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V56 M125,56h14l-14,14V56z M125,80l14-14 M125,90l14-14 M125,100l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot85",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V46 M125,46h14l-14,14V46z M125,70l14-14 M125,80l14-14 M125,90l14-14 M125,100l7-7 M125,125l7-12.1 h-14L125,125z\"/>",
        ),
        (
            "knot90",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V46 M125,46h14l-14,14V46z M125,70l14-14 M125,80l14-14 M125,90l14-14 M125,100l14-14 M125,125l7-12.1 h-14L125,125z\"/>",
        ),
        (
            "knot95",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V36 M125,36h14l-14,14V36z M125,60l14-14 M125,70l14-14 M125,80l14-14 M125,90l14-14 M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot100",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V62 M125,62h14l-14,14V62z M125,76h14l-14,14V76z M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot105",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V62 M125,62h14l-14,14V62z M125,76h14l-14,14V76z M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot110",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V62 M125,62h14l-14,14V62z M125,76h14l-14,14V76z M125,100l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot115",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V52 M125,52h14l-14,14V52z M125,66h14l-14,14V66z M125,90l14-14 M125,100l7-7 M125,125l7-12.1h-14 L125,125z\"/>",
        ),
        (
            "knot120",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V52 M125,52h14l-14,14V52z M125,66h14l-14,14V66z M125,90l14-14 M125,100l14-14 M125,125l7-12.1h-14 L125,125z\"/>",
        ),
        (
            "knot125",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V42 M125,42h14l-14,14V42z M125,56h14l-14,14V56z M125,80l14-14 M125,90l14-14 M125,100l7-7 M125,125 l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot130",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V42 M125,42h14l-14,14V42z M125,56h14l-14,14V56z M125,80l14-14 M125,90l14-14 M125,100l14-14 M125,125 l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot135",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V32 M125,32h14l-14,14V32z M125,46h14l-14,14V46z M125,70l14-14 M125,80l14-14 M125,90l14-14 M125,100 l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot140",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V32 M125,32h14l-14,14V32z M125,46h14l-14,14V46z M125,70l14-14 M125,80l14-14 M125,90l14-14 M125,100 l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot145",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V22 M125,22h14l-14,14V22z M125,36h14l-14,14V36z M125,60l14-14 M125,70l14-14 M125,80l14-14 M125,90 l14-14 M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot150",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V48 M125,48h14l-14,14V48z M125,62h14l-14,14V62z M125,76h14l-14,14V76z M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot155",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V48 M125,48h14l-14,14V48z M125,62h14l-14,14V62z M125,76h14l-14,14V76z M125,100l7-7 M125,125l7-12.1 h-14L125,125z\"/>",
        ),
        (
            "knot160",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V48 M125,48h14l-14,14V48z M125,62h14l-14,14V62z M125,76h14l-14,14V76z M125,100l14-14 M125,125 l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot165",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V38 M125,38h14l-14,14V38z M125,52h14l-14,14V52z M125,66h14l-14,14V66z M125,90l14-14 M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot170",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V38 M125,38h14l-14,14V38z M125,52h14l-14,14V52z M125,66h14l-14,14V66z M125,90l14-14 M125,100l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot175",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V28 M125,28h14l-14,14V28z M125,42h14l-14,14V42z M125,56h14l-14,14V56z M125,80l14-14 M125,90l14-14 M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot180",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V28 M125,28h14l-14,14V28z M125,42h14l-14,14V42z M125,56h14l-14,14V56z M125,80l14-14 M125,90l14-14 M125,100l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot185",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V18 M125,18h14l-14,14V18z M125,32h14l-14,14V32z M125,46h14l-14,14V46z M125,70l14-14 M125,80l14-14 M125,90l14-14 M125,100l7-7 M125,125l7-12.1h-14L125,125z\"/>",
        ),
        (
            "knot190",
            "<path class=\"svg-wb\" transform-origin=\"125 125\" transform=\"translate({TRANSLATE}) rotate({ROTATE}) scale({SCALE})\" d=\"M125,112V18 M125,18h14l-14,14V18z M125,32h14l-14,14V32z M125,46h14l-14,14V46z M125,70l14-14 M125,80l14-14 M125,90l14-14 M125,100l14-14 M125,125l7-12.1h-14L125,125z\"/>",
        ),
    ])
});

pub fn get_arrow_path(
    arrow_type: &ArrowType,
    magnitude_ms_s: f32,
    rotate: f32,
    translate: (f32, f32),
    scale: f32,
) -> String {
    let svg_path;
    match arrow_type {
        ArrowType::PivotCenter => {
            svg_path = ARROW_PATHS.get("arrow-pivot-center").unwrap().to_string();
        }
        ArrowType::PivotTip => {
            svg_path = ARROW_PATHS.get("arrow-pivot-tip").unwrap().to_string();
        }
        ArrowType::WindBarb => {
            let knots = magnitude_ms_s * 1.9438445;
            if knots < 1.0 {
                svg_path = ARROW_PATHS.get("knot0").unwrap().to_string();
            } else if knots < 3.5 {
                svg_path = ARROW_PATHS.get("knot2").unwrap().to_string();
            } else if knots >= 187.5 {
                svg_path = ARROW_PATHS.get("knot190").unwrap().to_string();
            } else {
                let rounded_knots = (knots / 5.0).round() * 5.0;
                let key = format!("knot{}", rounded_knots as i32);
                svg_path = ARROW_PATHS
                    .get(key.as_str())
                    .unwrap_or(ARROW_PATHS.get("knot0").unwrap())
                    .to_string();
            }
        }
    }
    svg_path
        .replace(
            "{TRANSLATE}",
            // all svg paths are centered at 125, 125, so for the center to end up at the give coordinates, 125 must be subtracted
            &format!("{:.2} {:.2}", translate.0 - 125.0, translate.1 - 125.0),
        )
        .replace("{ROTATE}", &format!("{:.2}", rotate))
        .replace("{SCALE}", &format!("{:.4}", scale))
}
