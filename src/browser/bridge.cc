#include "carbonyl/src/browser/bridge.h"

namespace {

float dpi_ = 0.0;
bool bitmap_mode_ = false;

}

namespace carbonyl {

void Bridge::Resize() {}

float Bridge::GetDPI() {
    return dpi_;
}

bool Bridge::BitmapMode() {
    return bitmap_mode_;
}

void Bridge::Configure(float dpi, bool bitmap_mode) {
    dpi_ = dpi;
    bitmap_mode_ = bitmap_mode;
}

}
