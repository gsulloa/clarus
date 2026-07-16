"use strict";

/**
 * Clarus landing page — progressive enhancement of the download region.
 *
 * The HTML already renders a working "Download" link (pointing at GitHub
 * Releases) with no JavaScript at all. This script only *upgrades* that
 * region once (if) the release manifest is available: it detects the
 * visitor's platform, fetches `download.json`, and swaps in the matching
 * installer, resolved version, and human-readable size. Any failure along
 * the way — network error, non-200, malformed JSON, missing platform key —
 * leaves (or restores) the GitHub Releases fallback. No code path should
 * ever leave a button pointing nowhere.
 */

(function () {
  var MANIFEST_URL = "https://releases.clarus.gulloa.click/download.json";
  var GITHUB_RELEASES_URL = "https://github.com/gsulloa/clarus/releases/latest";
  var FETCH_TIMEOUT_MS = 8000;

  var PLATFORM_LABELS = {
    "darwin-aarch64": "macOS (Apple Silicon)",
    "darwin-x86_64": "macOS (Intel)",
    "linux-x86_64": "Linux",
    "windows-x86_64": "Windows",
  };

  var SHORT_OS_LABEL = {
    "darwin-aarch64": "macOS",
    "darwin-x86_64": "macOS",
    "linux-x86_64": "Linux",
    "windows-x86_64": "Windows",
  };

  /** Detect a manifest platform key from the browser. Returns null if unknown. */
  function detectPlatform() {
    var ua = (navigator.userAgent || "").toLowerCase();
    var platform = (navigator.platform || "").toLowerCase();

    if (platform.indexOf("win") === 0 || ua.indexOf("windows") !== -1) {
      return "windows-x86_64";
    }

    if (platform.indexOf("mac") === 0 || ua.indexOf("mac os") !== -1 || ua.indexOf("macintosh") !== -1) {
      // Browsers cannot reliably distinguish Apple Silicon from Intel.
      // Default to Apple Silicon; the "All platforms" list always exposes
      // both Mac builds so Intel users can pick correctly.
      return "darwin-aarch64";
    }

    if (platform.indexOf("linux") === 0 || ua.indexOf("linux") !== -1) {
      // Avoid classifying Android (a Linux UA) as a desktop Linux build.
      if (ua.indexOf("android") !== -1) {
        return null;
      }
      return "linux-x86_64";
    }

    return null;
  }

  /** Convert a byte count into a human-readable KB/MB string. */
  function formatBytes(bytes) {
    if (typeof bytes !== "number" || !isFinite(bytes) || bytes < 0) {
      return "";
    }
    if (bytes < 1024) {
      return bytes + " B";
    }
    var kb = bytes / 1024;
    if (kb < 1024) {
      return round1(kb) + " KB";
    }
    var mb = kb / 1024;
    if (mb < 1024) {
      return round1(mb) + " MB";
    }
    var gb = mb / 1024;
    return round1(gb) + " GB";
  }

  function round1(n) {
    return Math.round(n * 10) / 10;
  }

  function fetchWithTimeout(url, timeoutMs) {
    if (typeof AbortController === "undefined") {
      return fetch(url, { cache: "no-cache" });
    }
    var controller = new AbortController();
    var timer = setTimeout(function () {
      controller.abort();
    }, timeoutMs);
    return fetch(url, { cache: "no-cache", signal: controller.signal }).finally(function () {
      clearTimeout(timer);
    });
  }

  /** Set every primary-CTA element (hero + final block) to the GitHub fallback. */
  function renderFallbackPrimary(reasonNote) {
    var primaries = document.querySelectorAll("[data-download-primary]");
    for (var i = 0; i < primaries.length; i++) {
      var el = primaries[i];
      el.setAttribute("href", GITHUB_RELEASES_URL);
      var label = el.querySelector("[data-download-label]");
      var meta = el.querySelector("[data-download-meta]");
      if (label) label.textContent = "Download from GitHub Releases";
      if (meta) meta.textContent = "All platforms, signed & notarized";
    }
    var notes = document.querySelectorAll("[data-download-status]");
    for (var j = 0; j < notes.length; j++) {
      notes[j].textContent = reasonNote || "";
    }
  }

  /** Render the matched installer as the primary CTA. */
  function renderPrimary(installer, platformKey, version) {
    var primaries = document.querySelectorAll("[data-download-primary]");
    var label = "Download for " + (SHORT_OS_LABEL[platformKey] || "your platform");
    var meta = "v" + version + " · " + formatBytes(installer.size);
    for (var i = 0; i < primaries.length; i++) {
      var el = primaries[i];
      el.setAttribute("href", installer.url);
      var labelEl = el.querySelector("[data-download-label]");
      var metaEl = el.querySelector("[data-download-meta]");
      if (labelEl) labelEl.textContent = label;
      if (metaEl) metaEl.textContent = meta;
    }
    var notes = document.querySelectorAll("[data-download-status]");
    for (var j = 0; j < notes.length; j++) {
      notes[j].textContent =
        platformKey === "darwin-aarch64"
          ? "Detected Apple Silicon Mac. Using Intel? Pick the x86_64 build below."
          : "";
    }
  }

  /** Populate the "All platforms" list from installers. Returns true if at least one row rendered. */
  function renderAllPlatforms(installers, version) {
    var container = document.getElementById("all-platforms");
    if (!container) return false;

    var keys = ["darwin-aarch64", "darwin-x86_64", "linux-x86_64", "windows-x86_64"];
    var rows = [];

    for (var i = 0; i < keys.length; i++) {
      var key = keys[i];
      var installer = installers[key];
      if (!installer || !installer.url) continue;

      var row = document.createElement("a");
      row.className = "platform-row";
      row.href = installer.url;

      var labelWrap = document.createElement("span");
      labelWrap.className = "platform-row-label";

      var name = document.createElement("span");
      name.className = "platform-row-name";
      name.textContent = PLATFORM_LABELS[key] || key;

      var filename = document.createElement("span");
      filename.className = "platform-row-filename";
      filename.textContent = installer.filename || "";

      labelWrap.appendChild(name);
      labelWrap.appendChild(filename);

      var size = document.createElement("span");
      size.className = "platform-row-size";
      size.textContent = formatBytes(installer.size);

      row.appendChild(labelWrap);
      row.appendChild(size);
      rows.push(row);
    }

    if (rows.length === 0) {
      return false;
    }

    container.textContent = "";
    for (var j = 0; j < rows.length; j++) {
      container.appendChild(rows[j]);
    }
    return true;
  }

  function renderFooterVersion(version) {
    var els = document.querySelectorAll("[data-footer-version]");
    for (var i = 0; i < els.length; i++) {
      els[i].textContent = version;
    }
  }

  function init() {
    var platformKey = detectPlatform();

    fetchWithTimeout(MANIFEST_URL, FETCH_TIMEOUT_MS)
      .then(function (response) {
        if (!response.ok) {
          throw new Error("Manifest request failed with status " + response.status);
        }
        return response.json();
      })
      .then(function (manifest) {
        if (!manifest || typeof manifest !== "object" || !manifest.installers || !manifest.version) {
          throw new Error("Manifest payload missing required fields");
        }

        var installers = manifest.installers;
        var version = manifest.version;
        var haveAnyPlatforms = renderAllPlatforms(installers, version);

        var matched = platformKey ? installers[platformKey] : null;
        if (matched && matched.url) {
          renderPrimary(matched, platformKey, version);
        } else {
          // Manifest loaded but the visitor's platform is absent (or unknown):
          // still show available builds, point the primary CTA at GitHub.
          renderFallbackPrimary(
            haveAnyPlatforms
              ? "No build detected for your platform — choose one below."
              : "",
          );
        }

        if (haveAnyPlatforms) {
          renderFooterVersion(version);
        }
      })
      .catch(function () {
        // Network error, non-200, bad JSON, or missing fields: keep the
        // static GitHub fallback that's already in the HTML, and make sure
        // both primary CTAs are consistent with it.
        renderFallbackPrimary("");
      });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
