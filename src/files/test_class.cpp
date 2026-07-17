#include <stdio.h>

class Pessoa
{
public:
    string nome;
    int idade;

    Pessoa(string n, int i)
    {
        nome = n;
        idade = i;
    }

    int getIdade()
    {
        return idade;
    }
};

int main()
{
    Pessoa p = Pessoa("Joao", 25);
    int x = p.getIdade();
    return 0;
}
