function findNextOrEqualTimestamp(
  timestamp: bigint,
  timestampArray: bigint[],
): bigint {
  // returns the first element form timestampArray that is bigger or equal to timestamp
  // We assume that timestampArray is sorted
  if (timestampArray.length == 0) {
    throw new Error("timestampArray is empty");
  }
  for (var i = 0; i < timestampArray.length; i++) {
    if (timestampArray[i] >= timestamp) {
      return timestampArray[i];
    }
  }
  // if all elements are smaller, return the last element
  return timestampArray.at(-1)!;
}

function findNextTimestamp(
  timestamp: bigint,
  timestampArray: bigint[],
): bigint {
  // returns the first element from timestampArray that is bigger than timestamp
  // We assume that timestampArray is sorted
  if (timestampArray.length == 0) {
    throw new Error("timestampArray is empty");
  }
  for (var i = 0; i < timestampArray.length; i++) {
    if (timestampArray[i] > timestamp) {
      return timestampArray[i];
    }
  }
  // if all elements are smaller, return the last element
  return timestampArray.at(-1)!;
}

function findPreviousTimestamp(
  timestamp: bigint,
  timestampArray: bigint[],
): bigint {
  // returns the biggest element from timestampArray that is smaller than timestamp
  // We assume that timestampArray is sorted
  if (timestampArray.length == 0) {
    throw new Error("timestampArray is empty");
  }
  for (var i = timestampArray.length - 1; i >= 0; i--) {
    if (timestampArray[i] < timestamp) {
      return timestampArray[i];
    }
  }
  // if all elements are bigger, return the first element
  return timestampArray.at(0)!;
}

export { findNextOrEqualTimestamp, findNextTimestamp, findPreviousTimestamp };
