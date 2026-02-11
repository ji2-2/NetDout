const statusEl = document.getElementById("status");

document.getElementById("queueBtn").addEventListener("click", () => {
  const url = document.getElementById("url").value.trim();
  const output = document.getElementById("output").value.trim() || "./download.bin";
  chrome.runtime.sendMessage({ type: "NETDOUT_QUEUE", url, output }, (resp) => {
    statusEl.textContent = JSON.stringify(resp, null, 2);
    if (resp?.ok && resp.data?.id) {
      document.getElementById("jobId").value = resp.data.id;
    }
  });
});

document.getElementById("statusBtn").addEventListener("click", () => {
  const id = document.getElementById("jobId").value.trim();
  chrome.runtime.sendMessage({ type: "NETDOUT_STATUS", id }, (resp) => {
    statusEl.textContent = JSON.stringify(resp, null, 2);
  });
});
