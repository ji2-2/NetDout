const DEFAULT_DAEMON = "http://127.0.0.1:8472";

async function getDaemonUrl() {
  const { daemonUrl } = await chrome.storage.local.get(["daemonUrl"]);
  return daemonUrl || DEFAULT_DAEMON;
}

async function queueDownload(url, output = "./download.bin") {
  const daemon = await getDaemonUrl();
  const response = await fetch(`${daemon}/downloads`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ url, output })
  });
  if (!response.ok) {
    throw new Error(`daemon error: ${response.status}`);
  }
  return response.json();
}

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type === "NETDOUT_QUEUE") {
    queueDownload(message.url, message.output)
      .then((data) => sendResponse({ ok: true, data }))
      .catch((error) => sendResponse({ ok: false, error: error.message }));
    return true;
  }

  if (message?.type === "NETDOUT_STATUS") {
    getDaemonUrl()
      .then((daemon) => fetch(`${daemon}/downloads/${message.id}`))
      .then((resp) => resp.json())
      .then((data) => sendResponse({ ok: true, data }))
      .catch((error) => sendResponse({ ok: false, error: error.message }));
    return true;
  }

  return false;
});
