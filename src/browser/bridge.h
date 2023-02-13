#ifndef CARBONYL_SRC_BROWSER_BRIDGE_H_
#define CARBONYL_SRC_BROWSER_BRIDGE_H_

#include "carbonyl/src/browser/export.h"

namespace carbonyl {

class CARBONYL_BRIDGE_EXPORT Bridge {
public:

  static void Resize();
  static bool BitmapMode();
  static float GetDPI();
};

}

#endif  // CARBONYL_SRC_BROWSER_BRIDGE_H_
