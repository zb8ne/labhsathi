const form = document.getElementById("profile-form");
const resultsSection = document.getElementById("results");
const resultsList = document.getElementById("results-list");
const extractBtn = document.getElementById("extract-btn");
const docInput = document.getElementById("doc-input");
const extractStatus = document.getElementById("extract-status");

extractBtn.addEventListener("click", async () => {
  const file = docInput.files[0];
  if (!file) {
    extractStatus.textContent = "Choose an image first.";
    return;
  }
  extractStatus.textContent = "Reading document (not stored)...";
  const fd = new FormData();
  fd.append("document", file);

  try {
    const res = await fetch("/api/extract-document", { method: "POST", body: fd });
    const data = await res.json();
    if (!res.ok) {
      extractStatus.textContent = "Could not read document: " + (data.error || "unknown error");
      return;
    }
    if (data.age) form.age.value = data.age;
    if (data.annual_income) form.annual_income.value = data.annual_income;
    if (data.state) form.state.value = data.state;
    if (data.category) form.category.value = data.category;
    if (data.land_holding_acres) form.land_holding_acres.value = data.land_holding_acres;
    extractStatus.textContent = "Fields filled from document. The image itself was discarded — nothing was saved.";
  } catch (e) {
    extractStatus.textContent = "Extraction failed: " + e.message;
  }
  // Clear the file input so the image doesn't linger in the browser form state either.
  docInput.value = "";
});

form.addEventListener("submit", async (e) => {
  e.preventDefault();
  const fd = new FormData(form);
  const profile = {
    age: Number(fd.get("age")),
    annual_income: Number(fd.get("annual_income")),
    occupation: fd.get("occupation"),
    state: fd.get("state"),
    gender: fd.get("gender"),
    has_disability: form.has_disability.checked,
    disability_percentage: fd.get("disability_percentage") ? Number(fd.get("disability_percentage")) : null,
    land_holding_acres: fd.get("land_holding_acres") ? Number(fd.get("land_holding_acres")) : null,
    family_size: Number(fd.get("family_size")),
    is_widow: form.is_widow.checked,
    category: fd.get("category"),
    is_student: form.is_student.checked,
    has_bank_account: form.has_bank_account.checked,
    has_kutcha_house: form.has_kutcha_house.checked,
    is_pregnant_or_lactating_first_child: form.is_pregnant_or_lactating_first_child.checked,
    girl_child_age: fd.get("girl_child_age") ? Number(fd.get("girl_child_age")) : null,
  };

  const res = await fetch("/api/match", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(profile),
  });
  const data = await res.json();

  resultsList.innerHTML = "";
  if (data.schemes.length === 0) {
    resultsList.innerHTML = "<p>No matches found among the schemes in this prototype's database — this isn't exhaustive, it demonstrates the matching approach.</p>";
  } else {
    for (const s of data.schemes) {
      const el = document.createElement("div");
      el.className = "scheme";
      el.innerHTML = `
        <h3>${s.name}</h3>
        <div class="authority">${s.authority}</div>
        <div class="benefit">${s.benefit}</div>
        <div class="reason">${s.reason}</div>
        <div class="docs"><strong>Documents needed:</strong> ${s.documents.join(", ")}</div>
        <div class="note">${s.official_note}</div>
      `;
      resultsList.appendChild(el);
    }
  }
  resultsSection.hidden = false;
  resultsSection.scrollIntoView({ behavior: "smooth" });
});
