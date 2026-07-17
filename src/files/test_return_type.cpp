#include <stdio.h>

int soma(int a, int b)
{
    return a + b;
}

int com_if(int x)
{
    if (x > 0)
    {
        return x;
    }
    return 0;
}

int erro()
{
    return "texto";
}

int main()
{
    int r = soma(10, 20);
    return 0;
}
