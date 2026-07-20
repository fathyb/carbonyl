#include "carbonyl/src/browser/download_manager_delegate.h"

#include <string>

#include "base/files/file_path.h"
#include "base/files/file_util.h"
#include "base/path_service.h"
#include "base/threading/thread_restrictions.h"
#include "carbonyl/src/browser/renderer.h"
#include "components/download/public/common/download_danger_type.h"
#include "components/download/public/common/download_interrupt_reasons.h"
#include "components/download/public/common/download_item.h"
#include "content/public/browser/download_item_utils.h"
#include "content/public/browser/web_contents.h"
#include "net/base/filename_util.h"
#include "url/origin.h"

namespace carbonyl {

DownloadManagerDelegate::DownloadManagerDelegate() = default;
DownloadManagerDelegate::~DownloadManagerDelegate() = default;

void DownloadManagerDelegate::GetNextId(content::DownloadIdCallback callback) {
  std::move(callback).Run(next_id_++);
}

bool DownloadManagerDelegate::DetermineDownloadTarget(
    download::DownloadItem* item,
    content::DownloadTargetCallback* callback) {
  base::ScopedAllowBlocking allow_blocking;

  base::FilePath downloads_dir;
  base::PathService::Get(base::DIR_HOME, &downloads_dir);
  downloads_dir = downloads_dir.AppendASCII("Downloads");
  base::CreateDirectory(downloads_dir);

  // Pick filename: suggested > URL-extracted > fallback
  std::string suggested = item->GetSuggestedFilename();
  base::FilePath filename;

  if (!suggested.empty()) {
    filename = base::FilePath::FromUTF8Unsafe(suggested);
  } else {
    filename = net::GenerateFileName(
        item->GetURL(),
        item->GetContentDisposition(),
        std::string(),
        std::string(),
        item->GetMimeType(),
        "download");
  }

  filename = filename.BaseName();
  if (filename.empty() || filename.value() == ".") {
    filename = base::FilePath::FromUTF8Unsafe("download");
  }

  base::FilePath target_path = downloads_dir.Append(filename);

  // Append counter to avoid clobbering existing files
  if (base::PathExists(target_path)) {
    std::string stem = filename.RemoveExtension().value();
    std::string ext = filename.Extension();
    for (int i = 1; i < 1000; ++i) {
      base::FilePath candidate = downloads_dir.AppendASCII(
          stem + " (" + std::to_string(i) + ")" + ext);
      if (!base::PathExists(candidate)) {
        target_path = candidate;
        break;
      }
    }
  }

  // Notify terminal: shows "Downloading: <filename>" in title area
  auto* renderer = Renderer::GetCurrent();
  if (renderer) {
    renderer->NotifyDownload(target_path.value(), -1LL);
  }

  // Intermediate file while in progress
  base::FilePath intermediate_path =
      target_path.AddExtension(FILE_PATH_LITERAL(".crdownload"));

  std::move(*callback).Run(
      target_path,
      download::DownloadItem::TARGET_DISPOSITION_OVERWRITE,
      download::DOWNLOAD_DANGER_TYPE_NOT_DANGEROUS,
      download::DownloadItem::InsecureDownloadStatus::SAFE,
      intermediate_path,
      base::FilePath(),  // display_name (empty = use target_path)
      item->GetMimeType(),
      download::DOWNLOAD_INTERRUPT_REASON_NONE);

  return true;
}

bool DownloadManagerDelegate::ShouldAutomaticallyOpenFile(
    const GURL& url,
    const base::FilePath& path) {
  return false;
}

bool DownloadManagerDelegate::ShouldCompleteDownload(
    download::DownloadItem* item,
    base::OnceClosure normal_complete) {
  auto* renderer = Renderer::GetCurrent();
  if (renderer) {
    renderer->NotifyDownload(item->GetTargetFilePath().value(),
                             item->GetReceivedBytes());
  }
  return true;
}

// Bypass the default async WebContents-validity check, which can race against
// the navigation-to-download interception and silently drop downloads.
void DownloadManagerDelegate::CheckDownloadAllowed(
    const content::WebContents::Getter& web_contents_getter,
    const GURL& url,
    const std::string& request_method,
    absl::optional<url::Origin> request_initiator,
    bool from_download_cross_origin_redirect,
    bool content_initiated,
    content::CheckDownloadAllowedCallback check_download_allowed_cb) {
  std::move(check_download_allowed_cb).Run(true);
}

}  // namespace carbonyl
