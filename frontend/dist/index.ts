
const form = document.getElementById("chatForm") as HTMLFormElement | null;
const userInput = document.getElementById("userInput") as HTMLInputElement | null;

let chatContainer = document.getElementById("chatContainer") as HTMLDivElement | null;

if (!chatContainer) {
    chatContainer = document.createElement("div");

    chatContainer.id = "chatContainer" as string;
    chatContainer.style = "20px" as string;
    chatContainer.style.marginTop = "20px" as string;
    chatContainer.style.textAlign = "left" as string;

    document.querySelector(".content")?.appendChild(chatContainer);
}

if (form && userInput) {
    form.addEventListener("submit", async (event: SubmitEvent) => {
        event.preventDefault();

        const prompt: string = userInput.value.trim();
        if (!prompt) {
            console.log("⚠️ Input is empty!");
            return;
        }

        const userMessage: HTMLParagraphElement = document.createElement("p");
        userMessage.className = "chatMessage userMessage";
        userMessage.innerHTML = `<strong>You:</strong> ${prompt}`;
        chatContainer?.appendChild(userMessage);

        userInput.value = "";

        try {
            const response = await fetch("/api/chat", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    sessionId: "default",
                    model: "mistral",
                    message: prompt
                }),
            });

            const data = await response.json();

            const botMessage: HTMLParagraphElement = document.createElement("p");
            botMessage.className = "chatMessage botMessage";
            botMessage.innerHTML = `<strong>Bot:</strong> ${data.reply ?? JSON.stringify(data)}`;
            chatContainer?.appendChild(botMessage);

            chatContainer?.scrollIntoView({ behavior: "smooth", block: "end" });
        } catch (err) {
            console.error("Failed to get a reply from Ollama:", err);
        }
    });
}
