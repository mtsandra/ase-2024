
def block_signals(input, blocksize):
    
    length = len(input)
    num_blocks = length // blocksize + 1
    blocks = []
    
    for i in range(num_blocks):
        blocks.append(input[i*blocksize:(i+1)*blocksize])
        if (i+1) * blocksize >= length:
            blocks.append(input[(i+1)*blocksize:])
    
    return blocks
        
    