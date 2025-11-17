// Native messaging host name - must match the manifest registration
const NATIVE_HOST = "com.gripdl.app";

interface DownloadMessage {
  url: string;
  cookies?: string;
  referrer?: string;
  user_agent?: string;
}

// Intercept downloads
browser.downloads.onCreated.addListener(async (downloadItem) => {
  try {
    // Cancel the original download
    await browser.downloads.cancel(downloadItem.id);

    // Get cookies for the download URL
    const cookies = await browser.cookies.getAll({ url: downloadItem.url });
    const cookieString = cookies.map((c) => `${c.name}=${c.value}`).join("; ");

    // Get referrer from download item
    const referrer = downloadItem.referrer || undefined;

    // Get user agent (we'll use the browser's default)
    const userAgent = navigator.userAgent;

    // Prepare message
    const message: DownloadMessage = {
      url: downloadItem.url,
      cookies: cookieString || undefined,
      referrer: referrer,
      user_agent: userAgent,
    };

    // Send to native app via native messaging
    await sendNativeMessage(message);
  } catch (error) {
    console.error("Failed to intercept download:", error);
  }
});

async function sendNativeMessage(message: DownloadMessage): Promise<void> {
  return new Promise((resolve, reject) => {
    const port = browser.runtime.connectNative(NATIVE_HOST);

    port.onMessage.addListener((response) => {
      if (response.success) {
        resolve();
      } else {
        reject(new Error(response.message || "Native app returned error"));
      }
    });

    port.onDisconnect.addListener(() => {
      if (browser.runtime.lastError) {
        reject(new Error(browser.runtime.lastError.message));
      } else {
        resolve();
      }
    });

    // Send message
    port.postMessage(message);
  });
}

// Listen for messages from content scripts (if needed)
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === "download") {
    sendNativeMessage(message)
      .then(() => sendResponse({ success: true }))
      .catch((error) => sendResponse({ success: false, error: error.message }));
    return true; // Keep channel open for async response
  }
});

console.log("GripDL extension loaded");

