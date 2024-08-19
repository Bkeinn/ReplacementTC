# Warning
It seems to work, but I am noticing some weird behavior that has not been fixed yet, but more importantly, it is highly unoptimized, and I think it is loading the whole
file into memory, which could lead to crashes, so be warned

# Replacement Text Compression
A very simple but surprisingly working Text compression algorithm.
## How does it work
It pretty simple, ASCII does have many character in it, that you would not need in your everyday Text,
so this algorithm just goes through your text, checks witch ones you haven't used, and then replaces your most commonly used two character combinations with this single ASCII character.
## Make it perform better
The more combinations can be replaced, the better the compression performance of the algorithm,
so it also offers you to give equivalents, which will replace all characters of a given text with it's equivalents and so freeing up more Space in the ASCII table
Some common examples are:
- A->a
- !->.
- '->"
# How to use
## Encoding
The absolute basic command
```
ReplacementTC encod --input ../smalltest.txt
```

The command I normally use with some equivalents
```
ReplacementTC encod --input ../smalltest.txt -e "A,a,B,b,C,c,D,d,E,e,F,f,G,g,H,h,I,i,J,j,K,k,L,l,M,m,N,n,O,o,P,p,Q,q,R,r,S,s,T,t,U,u,V,v,W,w,X,x,Y,y,Z,z,!,.,?,.,"
```
## Decoding
As the equivalents are already changed decoding is a lot more simple
```
ReplacementTC decode --input test
```
