const els = {};
let ready = false;
let pendingPayload = null;

window.__PIP_KANPE_RENDER__ = (payload) => {
  if (!ready) {
    pendingPayload = payload;
    return;
  }

  renderPip(payload);
};

window.addEventListener("DOMContentLoaded", init);

async function init() {
  bindElements();
  bindEvents();
  ready = true;
  if (pendingPayload) {
    renderPip(pendingPayload);
    pendingPayload = null;
  }
  await invokeDesktop("request_pip_snapshot").catch((error) => {
    console.warn("PiP snapshot request failed", error);
  });
}

function bindElements() {
  ["pip-shell", "pip-image", "pip-empty", "pip-controls", "pip-prev", "pip-label", "pip-next"].forEach((id) => {
    els[toCamel(id)] = document.getElementById(id);
  });
}

function bindEvents() {
  els.pipPrev.addEventListener("click", () => stepCard(-1));
  els.pipNext.addEventListener("click", () => stepCard(1));
}

async function stepCard(direction) {
  await invokeDesktop("step_pip_card", { direction }).catch((error) => {
    console.warn("PiP navigation failed", error);
  });
}

function renderPip(payload) {
  if (!payload || typeof payload !== "object") {
    showMessage("画像情報を受信できませんでした。");
    return;
  }

  document.title = payload.title || "PiP カンペ";
  applyControlClasses(payload.controls);

  const hasCard = Boolean(payload.hasCard && payload.imageSrc);
  els.pipImage.style.display = hasCard ? "block" : "none";
  els.pipEmpty.style.display = hasCard ? "none" : "grid";
  els.pipImage.classList.toggle("cover", payload.fitMode === "cover");
  els.pipShell.classList.toggle("separate", Boolean(payload.controls?.separate));
  els.pipShell.classList.toggle("auto-hide-controls", Boolean(payload.controls?.autoHide));

  if (hasCard) {
    els.pipImage.src = payload.imageSrc;
    els.pipImage.alt = payload.imageAlt || "";
    els.pipLabel.textContent = payload.label || "";
  } else {
    els.pipImage.removeAttribute("src");
    els.pipImage.alt = "";
    els.pipLabel.textContent = "";
    els.pipEmpty.textContent = payload.message || "表示できる画像がありません。";
  }

  const canNavigate = Boolean(payload.canNavigate);
  els.pipPrev.disabled = !canNavigate;
  els.pipNext.disabled = !canNavigate;
  updateButtonLabels(Boolean(payload.controls?.vertical));
}

function applyControlClasses(controls = {}) {
  const classes = [
    "small",
    "medium",
    "large",
    "horizontal",
    "vertical",
    "vertical-left",
    "vertical-right",
    "full-height-buttons",
    "top",
    "middle",
    "bottom",
    "background-solid",
    "background-translucent",
    "background-clear",
    "separate",
    "label-hidden",
  ];

  els.pipControls.classList.remove(...classes);
  els.pipControls.classList.add(
    controls.size || "medium",
    controls.placement || "horizontal",
    controls.position || "bottom",
    controls.background || "background-solid",
  );
  els.pipControls.classList.toggle("vertical", Boolean(controls.vertical));
  els.pipControls.classList.toggle("full-height-buttons", Boolean(controls.fullHeightButtons));
  els.pipControls.classList.toggle("separate", Boolean(controls.separate));
  els.pipControls.classList.toggle("label-hidden", Boolean(controls.labelHidden));
}

function updateButtonLabels(vertical) {
  els.pipPrev.textContent = vertical ? "↑" : "←";
  els.pipNext.textContent = vertical ? "↓" : "→";
}

function showMessage(message) {
  els.pipImage.style.display = "none";
  els.pipImage.removeAttribute("src");
  els.pipEmpty.style.display = "grid";
  els.pipEmpty.textContent = message;
  els.pipPrev.disabled = true;
  els.pipNext.disabled = true;
}

function invokeDesktop(command, payload = {}) {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return Promise.reject(new Error("Tauri API is unavailable"));
  }
  return invoke(command, payload);
}

function toCamel(value) {
  return value.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
}
