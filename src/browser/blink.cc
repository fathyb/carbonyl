#include "carbonyl/src/browser/blink.h"

namespace carbonyl {   
namespace blink {

namespace {

bool bitmap_mode = false;

}

bool BitmapMode() {
    return bitmap_mode;
}

void EnableBitmapMode() {
    bitmap_mode = true;
}

}
}
