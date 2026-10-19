(function () {
  const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  document.querySelectorAll(".reveal").forEach((el) => {
    el.classList.add("reveal-pending");
  });

  if (!prefersReduced && "IntersectionObserver" in window) {
    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add("reveal-visible");
            entry.target.classList.remove("reveal-pending");
            io.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.12, rootMargin: "0px 0px -40px 0px" },
    );
    document.querySelectorAll(".reveal").forEach((el) => io.observe(el));
  } else {
    document.querySelectorAll(".reveal").forEach((el) => {
      el.classList.add("reveal-visible");
      el.classList.remove("reveal-pending");
    });
  }

  function animateCount(el) {
    const target = parseFloat(el.getAttribute("data-count-to") || "0", 10);
    const decimals = parseInt(el.getAttribute("data-decimals") || "0", 10);
    const duration = 1200;
    const start = performance.now();
    function frame(now) {
      const t = Math.min(1, (now - start) / duration);
      const eased = 1 - Math.pow(1 - t, 3);
      const val = target * eased;
      el.textContent = decimals ? val.toFixed(decimals) : Math.round(val).toString();
      if (t < 1) requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
  }

  const gainsSection = document.querySelector(".example-gains");
  if (gainsSection && !prefersReduced) {
    const counterIo = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) return;
          gainsSection.querySelectorAll("[data-count-to]").forEach(animateCount);
          gainsSection.querySelectorAll(".chart-bar").forEach((bar) => {
            const h = bar.getAttribute("data-height");
            bar.setAttribute("height", h);
            bar.setAttribute("y", String(100 - Number(h)));
          });
          counterIo.disconnect();
        });
      },
      { threshold: 0.3 },
    );
    counterIo.observe(gainsSection);
  } else if (gainsSection) {
    gainsSection.querySelectorAll("[data-count-to]").forEach((el) => {
      const target = el.getAttribute("data-count-to");
      const decimals = el.getAttribute("data-decimals");
      el.textContent = decimals ? parseFloat(target).toFixed(parseInt(decimals, 10)) : target;
    });
    gainsSection.querySelectorAll(".chart-bar").forEach((bar) => {
      const h = bar.getAttribute("data-height");
      bar.setAttribute("height", h);
      bar.setAttribute("y", String(100 - Number(h)));
    });
  }

  const flowBar = document.getElementById("flow-bar");
  const flowSteps = document.querySelectorAll(".flow-step");
  if (flowBar && flowSteps.length && !prefersReduced) {
    const flowIo = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            flowBar.style.width = "100%";
            flowSteps.forEach((step, i) => {
              setTimeout(() => step.classList.add("flow-step-active"), i * 180);
            });
            flowIo.disconnect();
          }
        });
      },
      { threshold: 0.25 },
    );
    flowIo.observe(document.querySelector(".flow"));
  } else {
    flowSteps.forEach((s) => s.classList.add("flow-step-active"));
    if (flowBar) flowBar.style.width = "100%";
  }

  const statusEl = document.getElementById("demo-status");
  function updateDemoStatus() {
    const on = document.querySelectorAll(".toggle-demo .toggle-switch.is-on").length;
    if (statusEl) statusEl.textContent = `${on} tweak${on === 1 ? "" : "s"} selected · example profile`;
  }

  document.querySelectorAll("[data-demo-toggle] .toggle-switch").forEach((btn) => {
    btn.addEventListener("click", () => {
      const on = btn.classList.toggle("is-on");
      btn.setAttribute("aria-pressed", on ? "true" : "false");
      updateDemoStatus();
    });
  });
  updateDemoStatus();

  fetch("/download/BoostX-Setup.exe", { method: "HEAD" })
    .then((res) => {
      const meta = document.getElementById("installer-meta");
      if (!meta || !res.ok) return;
      const len = res.headers.get("content-length");
      if (len) {
        const mb = (Number(len) / (1024 * 1024)).toFixed(1);
        meta.textContent = `v0.5.0 · Installer ~${mb} MB`;
      } else {
        meta.textContent = "v0.5.0 · BoostX-Setup.exe";
      }
    })
    .catch(() => {
      const meta = document.getElementById("installer-meta");
      if (meta) meta.textContent = "v0.5.0 · Auto Optimize · CS2 benchmark";
    });
})();
