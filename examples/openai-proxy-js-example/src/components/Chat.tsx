"use client";
import OpenAI from "openai";
import "@/styles/chat.css";
import { useState } from "react";

const openai = new OpenAI({
  apiKey: "todo",
  baseURL: "http://localhost:4000/",
  dangerouslyAllowBrowser: true,
});

enum ChatMessageType {
  User,
  System,
  Assistant,
}
const messageTypeToOpenAI = (type: ChatMessageType) => {
  switch (type) {
    case ChatMessageType.Assistant:
      return "assistant";
    case ChatMessageType.User:
      return "user";
    case ChatMessageType.System:
      return "system";
  }
};
interface ChatMessageProps {
  type: ChatMessageType;
  text: string;
}
const ChatMessage = ({ type, text }: ChatMessageProps) => {
  // Don't show system messages for now.
  if (type === ChatMessageType.System) return null;

  const wrapperClassName = (() => {
    switch (type) {
      case ChatMessageType.Assistant:
        return "flex items-end";
      case ChatMessageType.User:
        return "flex items-end justify-end";
    }
  })();
  const messageClassName = (() => {
    switch (type) {
      case ChatMessageType.Assistant:
        return "flex flex-col space-y-2 text-xs max-w-xs mx-2 order-2 items-start";
      case ChatMessageType.User:
        return "flex flex-col space-y-2 text-xs max-w-xs mx-2 order-1 items-end";
    }
  })();
  const messageColorClassName = (() => {
    switch (type) {
      case ChatMessageType.Assistant:
        return "rounded-bl-none bg-gray-300 text-gray-600";
      case ChatMessageType.User:
        return "rounded-br-none bg-blue-600 text-white";
    }
  })();

  return (
    <div className="chat-message">
      <div className={wrapperClassName}>
        <div className={messageClassName}>
          <div>
            <span
              className={`px-4 py-2 rounded-lg inline-block ${messageColorClassName}`}
            >
              {text}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default function Chat() {
  const [messages, setMessages] = useState<ChatMessageProps[]>([
    {
      type: ChatMessageType.System,
      text: "Act as a helpful chat assistant.",
    },
  ]);
  const [loading, setLoading] = useState(false);
  const [prompt, setPrompt] = useState("");

  const handleSubmit = async () => {
    if (loading) return;
    setLoading(true);
    const newMessages = [
      ...messages,
      {
        type: ChatMessageType.User,
        text: prompt,
      },
    ];
    setMessages(newMessages);

    try {
      const { choices } = await openai.chat.completions.create({
        messages: newMessages.map((message) => ({
          role: messageTypeToOpenAI(message.type),
          content: message.text,
        })),
        model: "gpt-3.5-turbo-0613",
        // todo: use streaming...
        // stream: true,
      });
      const assistantMessage = choices[0].message.content!;
      setMessages([
        ...newMessages,
        {
          type: ChatMessageType.Assistant,
          text: assistantMessage,
        },
      ]);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
      setPrompt("");
    }
  };

  return (
    <div className="flex-1 p:2 sm:p-6 justify-between flex flex-col h-screen w-full">
      <div
        id="messages"
        className="flex flex-col space-y-4 p-3 overflow-y-auto scrollbar-thumb-blue scrollbar-thumb-rounded scrollbar-track-blue-lighter scrollbar-w-2 scrolling-touch"
      >
        {messages.map((message, index) => (
          <ChatMessage key={index} {...message} />
        ))}
        {messages.length <= 1 && <h3>Send a message to get started!</h3>}
      </div>
      <div className="border-t-2 border-gray-200 px-4 pt-4 mb-2 sm:mb-0">
        <div className="relative flex">
          <input
            type="text"
            placeholder="Write your message!"
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            className="w-full focus:outline-none focus:placeholder-gray-400 text-gray-600 placeholder-gray-600 pl-2 bg-gray-200 rounded-md py-3"
          />
          <div className="absolute right-0 items-center inset-y-0 hidden sm:flex">
            <button
              disabled={loading}
              onClick={handleSubmit}
              type="button"
              className="inline-flex items-center justify-center rounded-lg px-4 py-3 transition duration-500 ease-in-out text-white bg-blue-500 hover:bg-blue-400 focus:outline-none disabled:opacity-25"
            >
              <span className="font-bold">Send</span>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="currentColor"
                className="h-6 w-6 ml-2 transform rotate-90"
              >
                <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z"></path>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
