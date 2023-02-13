#include "carbonyl/src/browser/bridge.h"

extern "C" {

bool carbonyl_bridge_bitmap_mode();
float carbonyl_bridge_get_dpi();

}

namespace {

float dpi = -1;
int bitmap_mode = -1;

}

namespace carbonyl {

void Bridge::Resize() {
    dpi = -1;
}

float Bridge::GetDPI() {
    if (dpi == -1) {
        dpi = carbonyl_bridge_get_dpi();
    }

    return dpi;
}

bool Bridge::BitmapMode() {
    if (bitmap_mode == -1) {
        bitmap_mode = carbonyl_bridge_bitmap_mode();
    }

    return bitmap_mode == 1;
}

}
