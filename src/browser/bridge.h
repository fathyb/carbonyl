#ifndef CARBONYL_SRC_BROWSER_BRIDGE_H_
#define CARBONYL_SRC_BROWSER_BRIDGE_H_

#include "carbonyl/src/browser/export.h"

namespace carbonyl {

class Renderer;

class CARBONYL_BRIDGE_EXPORT Bridge {
public:
  static float GetDPI();
  static bool BitmapMode();

private:
  friend class Renderer;

  static void Resize();
  static void Configure(float dpi, bool bitmap_mode);
};

}

#endif  // CARBONYL_SRC_BROWSER_BRIDGE_H_
