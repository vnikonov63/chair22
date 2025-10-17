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

export default function App() {
  type Cell = { id: number; input: string; output: string };

  const nextId = useRef(1);
  const [cells, setCells] = useState<Cell[]>([
    { id: nextId.current++, input: "", output: "" },
  ]);

  const runCell = async (index: number) => {
    const cell = cells[index];
    if (!cell) return;

    try {
      const API = import.meta.env.VITE_API_BASE;
      const res = await fetch(`${API}/eval`, {
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
          i === index ? { ...c, output: resultText } : c
        );
        if (index === prev.length - 1) {
          next.push({ id: nextId.current++, input: "", output: "" });
        }
        return next;
      });
    } catch (err) {
      setCells((prev) =>
        prev.map((c, i) => (i === index ? { ...c, output: String(err) } : c))
      );
    }
  };

  const updateCellInput = (index: number, value: string) => {
    setCells((prev) =>
      prev.map((c, i) => (i === index ? { ...c, input: value } : c))
    );
  };

  return (
    <Container maxW="100%" py={8} px={6}>
      <VStack align="stretch" gap={6} fontFamily="monospace">
        {cells.map((cell, idx) => (
          <Box key={cell.id}>
            <Flex gap={3} alignItems="center">
              <Button
                colorScheme="blue"
                size="sm"
                onClick={() => runCell(idx)}
                minW="80px"
              >
                Run
              </Button>

              <Textarea
                value={cell.input}
                onChange={(e) => updateCellInput(idx, e.target.value)}
                placeholder="Enter expression…"
                resize="vertical"
                minH="60px"
                fontSize="md"
                flex={1}
                onKeyDown={(e) => {
                  if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
                    e.preventDefault();
                    runCell(idx);
                  }
                }}
              />
            </Flex>

            <Box
              bg="gray.800"
              color="whiteAlpha.900"
              p={4}
              borderRadius="md"
              boxShadow="sm"
              borderWidth="1px"
              as="pre"
              whiteSpace="pre-wrap"
              wordBreak="break-word"
              mt={3}
            >
              <Text m={0} fontSize="md" lineHeight="tall">
                {cell.output ? `Result: ${cell.output}` : ""}
              </Text>
            </Box>
          </Box>
        ))}
      </VStack>
    </Container>
  );
}
