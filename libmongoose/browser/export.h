#ifndef CARBONYL_SRC_BROWSER_BRIDGE_EXPORT_H_
#define CARBONYL_SRC_BROWSER_BRIDGE_EXPORT_H_

// CARBONYL_BRIDGE_EXPORT
#if defined(COMPONENT_BUILD)

#if defined(WIN32)

#if defined(CARBONYL_BRIDGE_IMPLEMENTATION)
#define CARBONYL_BRIDGE_EXPORT __declspec(dllexport)
#else
#define CARBONYL_BRIDGE_EXPORT __declspec(dllimport)
#endif

#else  // !defined(WIN32)

#if defined(CARBONYL_BRIDGE_IMPLEMENTATION)
#define CARBONYL_BRIDGE_EXPORT __attribute__((visibility("default")))
#else
#define CARBONYL_BRIDGE_EXPORT
#endif

#endif

#else  // !defined(COMPONENT_BUILD)

#define CARBONYL_BRIDGE_EXPORT

#endif

// CARBONYL_RENDERER_EXPORT
#if defined(COMPONENT_BUILD)

#if defined(WIN32)

#if defined(CARBONYL_RENDERER_IMPLEMENTATION)
#define CARBONYL_RENDERER_EXPORT __declspec(dllexport)
#else
#define CARBONYL_RENDERER_EXPORT __declspec(dllimport)
#endif

#else  // !defined(WIN32)

#if defined(CARBONYL_RENDERER_IMPLEMENTATION)
#define CARBONYL_RENDERER_EXPORT __attribute__((visibility("default")))
#else
#define CARBONYL_RENDERER_EXPORT
#endif

#endif

#else  // !defined(COMPONENT_BUILD)

#define CARBONYL_RENDERER_EXPORT

#endif

// CARBONYL_VIZ_EXPORT
#if defined(COMPONENT_BUILD)

#if defined(WIN32)

#if defined(CARBONYL_VIZ_IMPLEMENTATION)
#define CARBONYL_VIZ_EXPORT __declspec(dllexport)
#else
#define CARBONYL_VIZ_EXPORT __declspec(dllimport)
#endif

#else  // !defined(WIN32)

#if defined(CARBONYL_VIZ_IMPLEMENTATION)
#define CARBONYL_VIZ_EXPORT __attribute__((visibility("default")))
#else
#define CARBONYL_VIZ_EXPORT
#endif

#endif

#else  // !defined(COMPONENT_BUILD)

#define CARBONYL_VIZ_EXPORT

#endif

#endif  // CARBONYL_SRC_BROWSER_BRIDGE_EXPORT_H_
