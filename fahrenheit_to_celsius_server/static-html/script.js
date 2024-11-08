document.getElementById("convertButton").addEventListener("click", async () => {
    const fahrenheitInput = document.getElementById("fahrenheit").value;
    const response = await fetch(`/convert?fahrenheit=${fahrenheitInput}`);

    if (response.ok) {
        const data = await response.json();
        document.getElementById("result").textContent = `Celsius: ${data.celsius.toFixed(2)}`;
    } else {
        document.getElementById("result").textContent = "Error converting temperature.";
    }
});
