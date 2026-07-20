#ifndef CARBONYL_SRC_BROWSER_DOWNLOAD_MANAGER_DELEGATE_H_
#define CARBONYL_SRC_BROWSER_DOWNLOAD_MANAGER_DELEGATE_H_

#include "content/public/browser/download_manager_delegate.h"
#include "content/public/browser/web_contents.h"

namespace carbonyl {

// Handles file download requests: saves to ~/Downloads/ and notifies terminal.
class DownloadManagerDelegate : public content::DownloadManagerDelegate {
 public:
  DownloadManagerDelegate();
  ~DownloadManagerDelegate() override;

  // content::DownloadManagerDelegate:
  void GetNextId(content::DownloadIdCallback callback) override;
  bool DetermineDownloadTarget(
      download::DownloadItem* item,
      content::DownloadTargetCallback* callback) override;
  bool ShouldAutomaticallyOpenFile(const GURL& url,
                                   const base::FilePath& path) override;
  bool ShouldCompleteDownload(download::DownloadItem* item,
                              base::OnceClosure normal_complete) override;
  void CheckDownloadAllowed(
      const content::WebContents::Getter& web_contents_getter,
      const GURL& url,
      const std::string& request_method,
      absl::optional<url::Origin> request_initiator,
      bool from_download_cross_origin_redirect,
      bool content_initiated,
      content::CheckDownloadAllowedCallback check_download_allowed_cb) override;

 private:
  uint32_t next_id_ = 1;
};

}  // namespace carbonyl

#endif  // CARBONYL_SRC_BROWSER_DOWNLOAD_MANAGER_DELEGATE_H_
