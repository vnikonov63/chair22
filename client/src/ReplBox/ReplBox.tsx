import { useState, useRef } from "react";
import {
  Box,
  Button,
  Textarea,
  VStack,
  Text,
  Container,
  Flex,
} from "@chakra-ui/react";

export default function App({ replId }: { replId: number | null }) {
  type Cell = {
    id: number;
    input: string;
    output: string;
    resultRecieved: boolean;
  };

  const nextId = useRef(1);
  const textareas = useRef<Map<number, HTMLTextAreaElement | null>>(new Map());

  const LINE_HEIGHT = 20;
  const VERTICAL_PADDING = 16;
  const MIN_LINES = 1;
  const MAX_LINES = 20;

  const resizeTextareaById = (id: number, text?: string) => {
    const el = textareas.current.get(id) ?? null;
    if (!el) return;

    const lines = (text ?? el.value).split("\n").length;
    const target = Math.min(
      MAX_LINES * LINE_HEIGHT + VERTICAL_PADDING,
      Math.max(
        MIN_LINES * LINE_HEIGHT + VERTICAL_PADDING,
        lines * LINE_HEIGHT + VERTICAL_PADDING
      )
    );

    el.style.height = `${target}px`;
  };
  const [cells, setCells] = useState<Cell[]>([
    { id: nextId.current++, input: "", output: "", resultRecieved: false },
  ]);

  const runCell = async (index: number) => {
    const cell = cells[index];
    if (!cell) return;
    if (replId === null) return;

    try {
      const API = import.meta.env.VITE_API_BASE;
      const res = await fetch(`${API}/eval/${replId}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ text: cell.input }),
      });

      let resultText: string;
      if (!res.ok) {
        const text = await res.text();
        resultText = `Server error: ${res.status} ${text}`;
      } else {
        const data = await res.json();
        resultText = data.result ?? "";
      }

      setCells((prev) => {
        const next = prev.map((c, i) =>
          i === index ? { ...c, output: resultText, resultRecieved: true } : c
        );
        if (index === prev.length - 1) {
          next.push({
            id: nextId.current++,
            input: "",
            output: "",
            resultRecieved: false,
          });
        }
        return next;
      });
    } catch (err) {
      setCells((prev) =>
        prev.map((c, i) =>
          i === index ? { ...c, output: String(err), resultRecieved: true } : c
        )
      );
    }
  };

  const updateCellInput = (index: number, value: string) => {
    const id = cells[index]?.id;
    setCells((prev) =>
      prev.map((c, i) => (i === index ? { ...c, input: value } : c))
    );

    if (typeof id === "number") {
      setTimeout(() => resizeTextareaById(id, value), 0);
    }
  };

  return (
    <Container maxW="100%" py={8} px={6}>
      <VStack align="stretch" gap={6} fontFamily="monospace">
        {cells.map((cell, idx) => (
          <Box
            key={cell.id}
            borderWidth={cell.resultRecieved ? "1px" : "0"}
            borderColor="gray.200"
            borderRadius="md"
            p={cell.resultRecieved ? 2 : 0}
            bg={cell.resultRecieved ? "gray.600" : "transparent"}
          >
            <Flex gap={3} alignItems="center">
              {!cell.resultRecieved && (
                <Button
                  colorScheme="blue"
                  size="sm"
                  onClick={() => runCell(idx)}
                  minW="80px"
                  disabled={replId === null}
                >
                  Run
                </Button>
              )}

              {!cell.resultRecieved ? (
                <Textarea
                  ref={(el) => {
                    textareas.current.set(cell.id, el);
                  }}
                  value={cell.input}
                  onChange={(e) => updateCellInput(idx, e.target.value)}
                  onInput={(e) => {
                    const el = e.target as HTMLTextAreaElement;
                    el.style.height = "auto";
                    el.style.height = Math.min(el.scrollHeight, 800) + "px";
                  }}
                  placeholder="Enter expression…"
                  resize="none"
                  minH="60px"
                  maxH="400px"
                  fontSize="md"
                  flex={1}
                  onKeyDown={(e) => {
                    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
                      e.preventDefault();
                      runCell(idx);
                    }
                  }}
                />
              ) : (
                <Box
                  as="pre"
                  whiteSpace="pre-wrap"
                  wordBreak="break-word"
                  p={2}
                  bg="gray.850"
                  borderRadius="md"
                  borderWidth="0px"
                  minH="60px"
                  maxH="400px"
                  overflowY="auto"
                  fontSize="md"
                  flex={1}
                >
                  <Text m={0} lineHeight="tall">
                    {`> ${cell.input}`}
                  </Text>
                </Box>
              )}
            </Flex>

            {cell.output && (
              <Box
                bg="gray.850"
                color="whiteAlpha.900"
                p={2}
                borderRadius="md"
                borderWidth="0px"
                as="pre"
                whiteSpace="pre-wrap"
                wordBreak="break-word"
                mt={3}
              >
                <Text m={0} fontSize="md" lineHeight="tall">
                  {`Result: ${cell.output}`}
                </Text>
              </Box>
            )}
          </Box>
        ))}
      </VStack>
    </Container>
  );
}
