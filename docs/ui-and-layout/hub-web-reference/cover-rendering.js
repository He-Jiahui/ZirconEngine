(function () {
  const coverFiles = {
    elysium: "project-elysium.png",
    stellar: "project-stellar-outpost.png",
    sands: "project-sands-of-time.png",
    woods: "project-whispering-woods.png",
    neon: "project-neon-streets.png",
    prototype: "project-stellar-outpost.png",
  };

  function classToken(value) {
    return String(value).toLowerCase().replace(/[^a-z0-9_-]+/g, "-").replace(/^-+|-+$/g, "") || "default";
  }

  function projectCover(project, variant = "card") {
    const theme = classToken(project.cover);
    const size = classToken(variant);
    const fileName = coverFiles[theme] ?? coverFiles.elysium;
    return `
      <span class="project-cover ${size} cover-${theme}" aria-hidden="true">
        <img class="project-cover-image" src="../../../zircon_hub/assets/covers/reference/${fileName}" alt="" loading="eager">
      </span>`;
  }

  window.ZirconHubCover = Object.freeze({ projectCover });
})();
